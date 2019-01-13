use base64;
use common::{
    decrypt, encrypt, find_secret, generate_private_number, generate_public_number,
    read_json_from_socket, ClientState, EncryptionMethod, Message, NormalMessage,
    SendPublicNumberMessage,
};
use json;
use messages::{SendEncryptionMethodMessage, SendParamReqMessage};
use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, Handler, PollOpt, Sender, Token};
use std::collections::VecDeque;

pub struct ClientSocket {
    pub socket: TcpStream,
    name: String,
    state: ClientState,
    token: Token,
    event_loop_channel: Sender<(EventSet, Option<String>)>,
    messages_to_send: VecDeque<String>,
    p: u32,
    g: u32,
    private: u32,
    public: u32,
    secret: u32,
    encryption: EncryptionMethod,
}

impl Handler for ClientSocket {
    type Timeout = usize;
    type Message = (EventSet, Option<String>);

    fn ready(&mut self, event_loop: &mut EventLoop<ClientSocket>, _token: Token, events: EventSet) {
        debug!("Events: {:?}", events);

        if events.is_error() || events.is_hup() {
            error!("Connection error. Press enter to exit.");
            event_loop.shutdown();
            return;
        }

        match self.state {
            ClientState::NotConnected => {
                self.send_param_request().unwrap();
                self.state = ClientState::ParamReqSent;
                self.event_loop_channel
                    .send((EventSet::readable(), None))
                    .unwrap();
            }
            ClientState::ParamReqSent => {
                if events.is_readable() {
                    self.read_params().unwrap();
                    self.state = ClientState::ParamsReceived;
                    self.event_loop_channel
                        .send((EventSet::writable() | EventSet::readable(), None))
                        .unwrap();
                }
            }
            ClientState::ParamsReceived => {
                if events.is_writable() {
                    self.send_public_number().unwrap();
                    self.state = ClientState::ClientNumberSent;
                    self.event_loop_channel
                        .send((EventSet::readable(), None))
                        .unwrap();
                } else if events.is_readable() {
                    self.read_server_public().unwrap();
                    self.state = ClientState::ServerNumberSent;
                    self.event_loop_channel
                        .send((EventSet::writable(), None))
                        .unwrap();
                }
            }
            ClientState::ClientNumberSent => {
                if events.is_readable() {
                    self.read_server_public().unwrap();
                    if self.is_encryption_set() {
                        self.state = ClientState::NumbersExchanged;
                        self.event_loop_channel
                            .send((EventSet::writable(), None))
                            .unwrap();
                    } else {
                        self.state = ClientState::Connected;
                        self.event_loop_channel
                            .send((EventSet::readable(), None))
                            .unwrap();
                    }
                }
            }
            ClientState::ServerNumberSent => {
                if events.is_writable() {
                    self.send_public_number().unwrap();
                    if self.is_encryption_set() {
                        self.state = ClientState::NumbersExchanged;
                        self.event_loop_channel
                            .send((EventSet::writable(), None))
                            .unwrap();
                    } else {
                        self.state = ClientState::Connected;
                        self.event_loop_channel
                            .send((EventSet::readable(), None))
                            .unwrap();
                    }
                }
            }
            ClientState::NumbersExchanged => {
                if events.is_writable() {
                    self.secret = find_secret(self.public, self.private, self.p);
                    debug!("secret is {}", self.secret);
                    self.send_encryption_method().unwrap();
                    self.state = ClientState::Connected;
                    self.event_loop_channel
                        .send((EventSet::readable(), None))
                        .unwrap();
                }
            }
            ClientState::Connected => {
                if events.is_writable() && !self.messages_to_send.is_empty() {
                    self.send_msg().unwrap();
                    self.event_loop_channel
                        .send((EventSet::readable(), None))
                        .unwrap();
                } else if events.is_readable() {
                    self.read_message().unwrap();
                    self.event_loop_channel
                        .send((EventSet::readable(), None))
                        .unwrap();
                }
            }
        }
    }

    fn notify(
        &mut self,
        event_loop: &mut EventLoop<ClientSocket>,
        (events, msg_opt): (EventSet, Option<String>),
    ) {
        if msg_opt.is_some() {
            self.messages_to_send.push_back(msg_opt.unwrap());
        }

        debug!("notified with events: {:?}", events);
        event_loop
            .reregister(
                &self.socket,
                self.token,
                events | EventSet::hup() | EventSet::error(),
                PollOpt::edge() | PollOpt::oneshot(),
            ).unwrap();
    }
}

impl ClientSocket {
    pub fn new(
        socket: TcpStream,
        event_loop_channel: Sender<(EventSet, Option<String>)>,
        name: String,
        encryption: EncryptionMethod,
    ) -> ClientSocket {
        ClientSocket {
            socket: socket,
            state: ClientState::NotConnected,
            token: Token(0),
            event_loop_channel: event_loop_channel,
            messages_to_send: VecDeque::new(),
            name: name,
            p: 0,
            g: 0,
            private: 0,
            public: 0,
            secret: 0,
            encryption: encryption,
        }
    }

    fn send_msg(&mut self) -> Result<(), String> {
        let msg = self.messages_to_send.pop_front().unwrap();
        let encrypted_msg_bytes = encrypt(&msg, &self.encryption, &self.secret);
        let string_base64 = base64::encode(&encrypted_msg_bytes);
        debug!("Sending msg: {:?}", string_base64);
        try!(NormalMessage::new(self.name.clone(), string_base64).send(&mut self.socket));

        Ok(())
    }

    fn send_encryption_method(&mut self) -> Result<(), String> {
        info!("Sending encryption method");
        try!(SendEncryptionMethodMessage::new(self.encryption.clone()).send(&mut self.socket));
        Ok(())
    }

    fn read_server_public(&mut self) -> Result<(), String> {
        let json = try!(read_json_from_socket(&mut self.socket));
        debug!("received json: {:?}", json.dump());

        info!("Received server public number from server");

        self.public = json["b"].to_string().parse().unwrap();

        Ok(())
    }

    fn read_message(&mut self) -> Result<(), String> {
        let mut json = try!(read_json_from_socket(&mut self.socket));
        debug!("received json: {:?}", json.dump());

        let msg = json["msg"].to_string();
        let decoded_msg_bytes = base64::decode(&msg).unwrap();

        let decrypted_msg_string = decrypt(decoded_msg_bytes, &self.encryption, &self.secret);
        debug!("decrypted message: {:?}", decrypted_msg_string);

        json["msg"] = decrypted_msg_string.into();
        debug!("{:?}", json);

        display_msg_json(json);

        Ok(())
    }

    fn is_encryption_set(&self) -> bool {
        self.encryption != EncryptionMethod::None
    }

    fn send_param_request(&mut self) -> Result<(), String> {
        info!("Sending param request");
        try!(SendParamReqMessage.send(&mut self.socket));
        Ok(())
    }

    fn read_params(&mut self) -> Result<(), String> {
        let json = try!(read_json_from_socket(&mut self.socket));
        debug!("received json: {:?}", json.dump());
        info!("Received params from server");

        self.p = json["p"].to_string().parse().unwrap();
        self.g = json["g"].to_string().parse().unwrap();

        Ok(())
    }

    fn send_public_number(&mut self) -> Result<(), String> {
        info!("Sending public number to server");
        self.private = generate_private_number();
        let client_public_number = generate_public_number(self.p, self.g, self.private);
        try!(SendPublicNumberMessage::new("a", client_public_number).send(&mut self.socket));

        Ok(())
    }
}

fn display_msg_json(json: json::JsonValue) {
    println!(
        "{} >>> {}",
        json["from"].to_string(),
        json["msg"].to_string()
    );
}

use common::{
    decrypt, encrypt, find_secret, generate_private_number, generate_public_number,
    read_json_from_socket, ClientState, EncryptionMethod, Message, NormalMessage,
    SendPublicNumberMessage,
};
use json;
use messages::SendParamsMessage;
use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, Sender, Token};
use parameters::generate_parameters;
use socket_server::SocketServer;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::rc::Rc;

pub struct SocketClient {
    pub token: Token,
    pub socket: TcpStream,
    address: SocketAddr,
    state: ClientState,
    messages_to_send: VecDeque<Box<Message>>,
    event_loop_notifier: Sender<(Token, EventSet)>,
    messages_to_broadcast: Rc<RefCell<VecDeque<(Token, json::JsonValue)>>>,
    p: u32,
    g: u32,
    private: u32,
    public: u32,
    secret: u32,
    encryption: EncryptionMethod,
}

impl SocketClient {
    pub fn new(
        token: Token,
        socket: TcpStream,
        addr: SocketAddr,
        event_loop_notifier: Sender<(Token, EventSet)>,
        messages_to_broadcast: Rc<RefCell<VecDeque<(Token, json::JsonValue)>>>,
    ) -> SocketClient {
        SocketClient {
            token: token,
            socket: socket,
            address: addr,
            state: ClientState::NotConnected,
            messages_to_send: VecDeque::new(),
            event_loop_notifier: event_loop_notifier,
            messages_to_broadcast: messages_to_broadcast,
            p: 0,
            g: 0,
            private: 0,
            public: 0,
            secret: 0,
            encryption: EncryptionMethod::None,
        }
    }

    pub fn handle_event(&mut self, _event_loop: &mut EventLoop<SocketServer>, events: EventSet) {
        if events.is_writable() && !self.messages_to_send.is_empty() {
            self.messages_to_send
                .pop_front()
                .unwrap()
                .send(&mut self.socket)
                .unwrap();

            match self.state {
                ClientState::ParamReqSent => {
                    info!("Params sent to client {:?}", self.token);
                    self.state = ClientState::ParamsReceived;
                    self.push_send_public_message();
                    self.reregister(EventSet::writable() | EventSet::readable());
                }
                ClientState::ParamsReceived => {
                    info!("Public number sent to client {:?}", self.token);
                    self.state = ClientState::ServerNumberSent;
                    self.reregister(EventSet::readable());
                }
                ClientState::ServerNumberSent => unreachable!(),
                ClientState::ClientNumberSent => {
                    info!("Public number sent to client {:?}", self.token);
                    self.state = ClientState::NumbersExchanged;
                    self.reregister(EventSet::readable());
                }
                ClientState::Connected => {
                    if self.messages_to_send.is_empty() {
                        self.reregister(EventSet::readable());
                    } else {
                        self.reregister(EventSet::writable());
                    }
                }
                _ => (),
            }
        }

        if events.is_readable() {
            match self.state {
                ClientState::NotConnected => {
                    self.read_param_req().unwrap();
                    self.reregister(EventSet::writable());
                }
                ClientState::ParamReqSent => unreachable!(),
                ClientState::ParamsReceived => {
                    self.read_public().unwrap();
                    self.state = ClientState::ClientNumberSent;
                    self.reregister(EventSet::writable());
                }
                ClientState::ClientNumberSent => unreachable!(),
                ClientState::ServerNumberSent => {
                    self.read_public().unwrap();
                    self.state = ClientState::NumbersExchanged;
                    self.reregister(EventSet::readable());
                }
                ClientState::NumbersExchanged => {
                    self.secret = find_secret(self.public, self.private, self.p);
                    debug!("secret is: {}", self.secret);
                    info!("secret established");
                    self.read_encryption_or_msg().unwrap();
                    self.state = ClientState::Connected;
                    self.reregister(EventSet::readable());
                }
                ClientState::Connected => {
                    self.read_message().unwrap();
                    self.reregister(EventSet::readable());
                }
            }
        }
    }

    pub fn push_normal_message_json(&mut self, json: json::JsonValue) {
        let from: String = json["from"].to_string();
        let msg: String = json["msg"].to_string();
        let msg_encrypted: String;

        if self.encryption != EncryptionMethod::None {
            msg_encrypted = encrypt(&msg, &self.encryption, &self.secret);
        } else {
            msg_encrypted = msg.clone();
        }
        debug!("Encrypted message: {:?}", msg_encrypted);

        info!(
            "Enqueue message from {}: {} to client {:?}",
            from, msg, self.token
        );
        self.messages_to_send.push_back(Box::new(NormalMessage::new(
            from.clone(),
            msg_encrypted.clone(),
        )));
    }

    fn read_encryption_or_msg(&mut self) -> Result<(), String> {
        let json = try!(read_json_from_socket(&mut self.socket));
        if is_valid_encryption_method(&json) {
            debug!("valid encryption req");
            self.handle_encryption(json);
        } else if is_valid_msg(&json) {
            debug!("valid msg");
            self.handle_msg(json);
        } else {
            return Err(String::from("Incorrect encryption or message"));
        }

        Ok(())
    }

    fn read_message(&mut self) -> Result<(), String> {
        let json = try!(read_json_from_socket(&mut self.socket));
        if is_valid_msg(&json) {
            debug!("valid msg");
            self.handle_msg(json);
        } else {
            return Err(String::from("Incorrect message"));
        }

        Ok(())
    }

    fn handle_encryption(&mut self, json: json::JsonValue) {
        info!(
            "received encryption method req from client: {:?}",
            self.token
        );

        match json["encryption"].to_string().as_ref() {
            "xor" => self.encryption = EncryptionMethod::Xor,
            "cezar" => self.encryption = EncryptionMethod::Cezar,
            _ => (),
        }
    }

    fn handle_msg(&mut self, mut json: json::JsonValue) {
        info!("received normal message from client: {:?}", self.token);
        debug!("{:?}", json);

        let msg = json["msg"].to_string();
        let decrypted_msg = decrypt(&msg, &self.encryption, &self.secret);
        debug!("decrypted message: {:?}", decrypted_msg);

        json["msg"] = decrypted_msg.into();
        debug!("{:?}", json);

        self.messages_to_broadcast
            .borrow_mut()
            .push_back((self.token, json));
        self.event_loop_notifier
            .send((Token(0), EventSet::writable()))
            .unwrap();
    }

    fn read_param_req(&mut self) -> Result<(), String> {
        let json = try!(read_json_from_socket(&mut self.socket));
        if validate_param_req(&json) {
            info!("Param request received from: {:?}", self.address);
            self.state = ClientState::ParamReqSent;
            let (p, g) = generate_parameters();
            self.p = p;
            self.g = g;
            self.messages_to_send
                .push_back(Box::new(SendParamsMessage::new(p, g)));
        } else {
            return Err(String::from("Could not read valid param request"));
        }
        Ok(())
    }

    fn push_send_public_message(&mut self) {
        self.private = generate_private_number();
        let server_public_number = generate_public_number(self.p, self.g, self.private);
        self.messages_to_send
            .push_back(Box::new(SendPublicNumberMessage::new(
                "b",
                server_public_number,
            )));
    }

    fn read_public(&mut self) -> Result<(), String> {
        let json = try!(read_json_from_socket(&mut self.socket));
        info!("Received public number from client: {:?}", self.token);
        self.public = json["a"].to_string().parse().unwrap();

        Ok(())
    }

    pub fn reregister(&mut self, event_set: EventSet) {
        debug!("reregister token {:?} {:?}", self.token, event_set);
        self.event_loop_notifier
            .send((self.token, event_set))
            .unwrap();
    }
}

fn validate_param_req(json: &json::JsonValue) -> bool {
    let param_req_json = object!{
        "request" => "keys"
    };
    debug!("received param req json: {:?}", param_req_json);
    debug!("received param req json: {:?}", json);

    return param_req_json == *json;
}

fn is_valid_encryption_method(json: &json::JsonValue) -> bool {
    debug!("is encryption method {}", json["encryption"].to_string());
    !json["encryption"].is_null()
}

fn is_valid_msg(json: &json::JsonValue) -> bool {
    !json["msg"].is_null() && !json["from"].is_null()
}

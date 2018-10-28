use bytes::ByteBuf;
use common::{Message, read_json_from_socket, ClientState};
use json;
use messages::SendParamsMessage;
use mio::tcp::TcpStream;
use mio::{Sender, Token};
use std::collections::VecDeque;
use std::net::SocketAddr;

pub struct SocketClient {
    pub token: Token,
    pub socket: TcpStream,
    address: SocketAddr,
    state: ClientState,
    messages_to_send: VecDeque<Box<Message>>,
    event_loop_notifier: Sender<Token>,
}

impl SocketClient {
    pub fn new(
        token: Token,
        socket: TcpStream,
        addr: SocketAddr,
        event_loop_notifier: Sender<Token>,
    ) -> SocketClient {
        SocketClient {
            token: token,
            socket: socket,
            address: addr,
            state: ClientState::NotConnected,
            messages_to_send: VecDeque::new(),
            event_loop_notifier: event_loop_notifier,
        }
    }

    pub fn has_messages_to_sent(&self) -> bool {
        !self.messages_to_send.is_empty()
    }

    pub fn read(&mut self) {
        let json = read_json_from_socket(&mut self.socket).unwrap(); //TODO wrap

        match self.state {
            ClientState::NotConnected => self.handle_param_req(json),
            _ => (),
        }
    }

    pub fn write(&mut self) {
        debug!(
            "Messages size of socket {:?}: {:?}",
            self.address,
            self.messages_to_send.len()
        );
        if !self.messages_to_send.is_empty() {
            info!("Sending message to: {:?}", self.address);
            //TODO ten pop napewno trzeba sprawdzić
            self.messages_to_send
                .pop_front()
                .unwrap()
                .send(&mut self.socket)
                .unwrap();  //TODO wrap
        }
    }

    fn handle_incoming_number(&mut self, data: &mut ByteBuf) {
        unimplemented!()
        //TODO in progress
    }

    fn handle_param_req(&mut self, json: json::JsonValue) {
        if validate_param_req(&json) {
            info!("Param request received from: {:?}", self.address);
            self.state = ClientState::ParamReqSent;
            let (p, g) = generate_parameters();
            self.messages_to_send
                .push_back(Box::new(SendParamsMessage::new(p, g)));
            self.event_loop_notifier.send(self.token).unwrap(); //TODO
        } else {
            unimplemented!() //TODO
        }
    }
}

fn generate_parameters() -> (u32, u32) {
    (5, 13) //TODO generate parameters
}

fn validate_param_req(json: &json::JsonValue) -> bool {
    let param_req_json = object!{
        "request" => "keys"
    };
    debug!("received param req json: {:?}", param_req_json);
    debug!("received param req json: {:?}", json);

    return param_req_json == *json;
}

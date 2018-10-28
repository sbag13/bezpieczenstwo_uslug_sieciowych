use bytes::ByteBuf;
use common::{decode_to_string, ClientState};
use json;
use messages::{Message, SendParamsMessage};
use mio::tcp::TcpStream;
use mio::TryRead;
use std::collections::VecDeque;
use std::net::SocketAddr;

pub struct SocketClient {
    pub socket: TcpStream,
    address: SocketAddr,
    state: ClientState,
    messages_to_send: VecDeque<Box<Message>>,
}

impl SocketClient {
    pub fn new(socket: TcpStream, addr: SocketAddr) -> SocketClient {
        SocketClient {
            socket: socket,
            address: addr,
            state: ClientState::NotConnected,
            messages_to_send: VecDeque::new(),
        }
    }

    pub fn read(&mut self) {
        loop {
            debug!("Trying to read from: {:?}", self.address);
            let mut mut_buf = ByteBuf::mut_with_capacity(2048);
            match self.socket.try_read_buf(&mut mut_buf) {
                Err(e) => {
                    error!("Error while reading socket: {:?}", e);
                    return;
                }
                Ok(None) =>
                // buff has no more bytes
                {
                    break;
                }
                Ok(Some(_length)) => {
                    debug!("data received: {:?}", mut_buf);
                    self.handle_data(&mut mut_buf.flip());
                    break;
                }
            }
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
                .send(&mut self.socket);
        }
    }

    fn handle_data(&mut self, data: &mut ByteBuf) {
        match self.state {
            ClientState::NotConnected => self.handle_param_req(data),
            _ => (),
        }
    }

    fn handle_incoming_number(&mut self, data: &mut ByteBuf) {
        let decoded_string = decode_to_string(data);
        unimplemented!()
        //TODO in progress
    }

    fn handle_param_req(&mut self, data: &mut ByteBuf) {
        if validate_param_req(&decode_to_string(data)) {
            info!("Param request received from: {:?}", self.address);
            self.state = ClientState::ParamReqSent;
            let (p, g) = generate_parameters();
            self.messages_to_send
                .push_back(Box::new(SendParamsMessage::new(p, g)));
        } else {
            unimplemented!()
        }
    }
}

fn generate_parameters() -> (u32, u32) {
    (5, 13) //TODO generate parameters
}

fn validate_param_req(decoded: &String) -> bool {
    let param_req_json = object!{
        "request" => "keys"
    };
    debug!("received param req json: {:?}", param_req_json);

    let parsed_decoded = match json::parse(decoded) {
        Ok(json) => json,
        Err(err) => {
            error!("{}", err);
            return false;
        }
    };
    debug!("received param req json: {:?}", parsed_decoded);

    return param_req_json == parsed_decoded;
}

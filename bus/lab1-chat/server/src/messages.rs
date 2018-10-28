use common::send_json_to_socket;
use json;
use mio::tcp::TcpStream;

pub trait Message {
    fn send(&self, socket: &mut TcpStream) {
        let json = self.create_json();
        send_json_to_socket(socket, json);
        info!("Message sent"); //TODO result itd
    }

    fn create_json(&self) -> json::JsonValue;
}

#[derive(Debug)]
pub struct SendParamsMessage {
    p: u32,
    g: u32,
}

impl Message for SendParamsMessage {
    fn create_json(&self) -> json::JsonValue {
        object!{
            "p" => self.p,
            "g" => self.g
        }
    }
}

impl SendParamsMessage {
    pub fn new(p: u32, g: u32) -> SendParamsMessage {
        SendParamsMessage { p: p, g: g }
    }
}

// #[derive(Debug)]
// struct SendPublicNumber {
//     b: u32,
// }

// #[derive(Debug)]
// struct SendRegularMessage {
//     msg: String,
// }

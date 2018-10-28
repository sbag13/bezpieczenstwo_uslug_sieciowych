extern crate base64;
extern crate mio;
#[macro_use]
extern crate log;
extern crate bytes;
extern crate json;

use bytes::ByteBuf;
use mio::tcp::TcpStream;
use mio::TryWrite;
use std::io::Read;

#[derive(Debug, PartialEq)]
pub enum ClientState {
    NotConnected,
    ParamReqSent,
}

pub fn send_json_to_socket(socket: &mut TcpStream, json: json::JsonValue) {
    let json_string = json.dump();
    debug!("param json string: {:?}", json_string);
    let string_base64 = base64::encode(json.dump().as_bytes());
    debug!("string in base64: {:?}", string_base64);
    socket.try_write(string_base64.as_bytes()).unwrap();
}

pub fn generate_public_number() -> u32 {
    return 5; //TODO
}

pub fn decode_to_string(data: &mut ByteBuf) -> String {
    let mut string_buf = String::new();
    data.read_to_string(&mut string_buf).unwrap();
    debug!("base64 string: {:?}", string_buf);

    let bytes_decoded = base64::decode(&string_buf).unwrap();
    debug!("decoded raw bytes: {:?}", bytes_decoded);

    let string_decoded = String::from(std::str::from_utf8(&bytes_decoded).unwrap());
    debug!("string decoded: {:?}", string_decoded);

    string_decoded
}

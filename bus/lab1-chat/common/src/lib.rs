extern crate base64;
extern crate mio;
#[macro_use]
extern crate log;
extern crate bytes;
#[macro_use]
extern crate json;

use bytes::ByteBuf;
use mio::tcp::TcpStream;
use mio::{TryRead, TryWrite};
use std::io::Read;

pub trait Message {
    fn send(&self, socket: &mut TcpStream) -> Result<(), String> {
        let json = self.create_json();
        try!(send_json_to_socket(socket, json));
        Ok(())
    }

    fn create_json(&self) -> json::JsonValue;
    fn get_msg_name(&self) -> String;
}

#[derive(Debug)]
pub struct SendPublicNumberMessage {
    key: String,
    val: u32,
}

impl Message for SendPublicNumberMessage {
    fn create_json(&self) -> json::JsonValue {
        object!{
            &self.key => self.val,
        }
    }

    fn get_msg_name(&self) -> String {
        return String::from(format!(
            "SendPublicNumberMessage ({}:{})",
            self.key, self.val
        ));
    }
}

impl SendPublicNumberMessage {
    pub fn new(key: &str, val: u32) -> SendPublicNumberMessage {
        SendPublicNumberMessage {
            key: String::from(key),
            val: val,
        }
    }
}


#[derive(Debug)]
pub struct NormalMessage {
    content: String,
    sender: String,
}

impl NormalMessage {
    pub fn new(sender: String, content: String) -> NormalMessage {
        NormalMessage { content: content, sender: sender }
    }
}

impl Message for NormalMessage {
    fn create_json(&self) -> json::JsonValue {
        object!{
            &self.sender => self.content.clone(),
        }
    }

    fn get_msg_name(&self) -> String {
        return String::from("NormalMessage");
    }
}

#[derive(Debug, PartialEq)]
pub enum EncryptionMethod {
    None,
    Xor,
    Cezar,
}

#[derive(Debug, PartialEq)]
pub enum ClientState {
    NotConnected,
    ParamReqSent,
    ParamsReceived,
    ClientNumberSent,
    ServerNumberSent,
    NumbersExchanged,
    Connected,
}

pub fn send_json_to_socket(socket: &mut TcpStream, json: json::JsonValue) -> Result<(), String> {
    let json_string = json.dump();
    debug!("sending json string: {:?}", json_string);
    let string_base64 = base64::encode(json.dump().as_bytes());
    debug!("string in base64: {:?}", string_base64);
    match socket.try_write(string_base64.as_bytes()) {
        Ok(_) => return Ok(()),
        Err(e) => return Err(e.to_string()),
    }
}

pub fn generate_public_number() -> u32 {
    return 5; //TODO
}

pub fn decode_to_string(data: &mut ByteBuf) -> String {
    let mut string_buf = String::new();
    data.read_to_string(&mut string_buf).unwrap();
    debug!("base64 string: {:?}", string_buf);

    let bytes_decoded = base64::decode(&string_buf).unwrap();
    debug!("decoded raw bytes: {:?}", bytes_decoded); //TODO maybe wrap it

    let string_decoded = String::from(std::str::from_utf8(&bytes_decoded).unwrap());
    debug!("string decoded: {:?}", string_decoded);

    string_decoded
}

pub fn get_data_from_socket(socket: &mut TcpStream) -> Result<Option<ByteBuf>, String> {
    loop {
        debug!("Trying to read from: {:?}", socket);
        let mut mut_buf = ByteBuf::mut_with_capacity(2048);
        match socket.try_read_buf(&mut mut_buf) {
            Err(e) => {
                error!("Error while reading socket: {:?}", e);
                return Err(e.to_string());
            }
            Ok(None) =>
            // buff has no more bytes
            {
                return Ok(None);
            }
            Ok(Some(_length)) => {
                debug!("data received: {:?}", mut_buf);
                return Ok(Some(mut_buf.flip()));
            }
        }
    }
}

pub fn read_json_from_socket(socket: &mut TcpStream) -> Result<json::JsonValue, String> {
    let mut data = match get_data_from_socket(socket) {
        Ok(Some(buf)) => buf,
        Ok(None) => {
            return Err(String::from("No data read from socket"));
        }
        Err(s) => return Err(s),
    };

    let string_decoded = decode_to_string(&mut data);

    match json::parse(&string_decoded) {
        Ok(json) => return Ok(json),
        Err(_) => return Err(String::from("Could not parse data into jason")),
    }
}

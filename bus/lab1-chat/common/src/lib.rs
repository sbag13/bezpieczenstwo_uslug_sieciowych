extern crate base64;
extern crate mio;
#[macro_use]
extern crate log;
extern crate bytes;
#[macro_use]
extern crate json;
extern crate byteorder;
extern crate rand;

use byteorder::{BigEndian, WriteBytesExt};
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
        NormalMessage {
            content: content,
            sender: sender,
        }
    }
}

impl Message for NormalMessage {
    fn create_json(&self) -> json::JsonValue {
        object!{
            "msg" => self.content.clone(),
            "from" => self.sender.clone(),
        }
    }

    fn get_msg_name(&self) -> String {
        return String::from("NormalMessage");
    }
}

pub fn encrypt(msg: &String, method: &EncryptionMethod, secret: &u32) -> String {
    match method {
        EncryptionMethod::Cezar => return encrypt_cezar(msg, secret),
        EncryptionMethod::Xor => return crypt_xor(msg, secret),
        _ => msg.clone(),
    }
}

pub fn decrypt(msg: &String, method: &EncryptionMethod, secret: &u32) -> String {
    match method {
        EncryptionMethod::Cezar => return decrypt_cezar(msg, secret),
        EncryptionMethod::Xor => return crypt_xor(msg, secret),
        _ => msg.clone(),
    }
}

fn crypt_xor(msg: &String, secret: &u32) -> String {
    let mut secret_bytes = vec![];
    secret_bytes.write_u32::<BigEndian>(*secret).unwrap();
    let encrypted_msg_bytes: Vec<u8> = msg
        .as_bytes()
        .iter()
        .map(|byte| byte ^ secret_bytes.iter().last().unwrap())
        .collect();

    String::from_utf8(encrypted_msg_bytes).unwrap()
}

fn encrypt_cezar(msg: &String, secret: &u32) -> String {
    let encrypted_msg_bytes: Vec<u8> = msg
        .as_bytes()
        .iter()
        .map(|byte| byte.wrapping_add(*secret as u8) % 128)
        .collect();

    String::from_utf8(encrypted_msg_bytes).unwrap()
}

fn decrypt_cezar(msg: &String, secret: &u32) -> String {
    let encrypted_msg_bytes: Vec<u8> = msg
        .as_bytes()
        .iter()
        .map(|byte| byte.wrapping_sub(*secret as u8) % 128)
        .collect();

    String::from_utf8(encrypted_msg_bytes).unwrap()
}

#[derive(Debug, PartialEq, Clone)]
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

pub fn generate_private_number() -> u32 {
    (rand::random::<f32>() * 20.0) as u32 + 2
}

pub fn generate_public_number(p: u32, g: u32, a: u32) -> u32 {
    debug!("g: {}, a: {}", g, a);
    let g_tmp = g as u128;
    (g_tmp.pow(a) % p as u128) as u32
}

pub fn find_secret(public: u32, private: u32, p: u32) -> u32 {
    debug!(
        "finding secret, public {}, private {}, p {}",
        public, private, p
    );
    let public_u128 = public as u128;
    (public_u128.pow(private) % p as u128) as u32
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

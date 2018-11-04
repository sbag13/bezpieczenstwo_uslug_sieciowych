extern crate mio;
#[macro_use]
extern crate log;
extern crate bytes;
#[macro_use]
extern crate json;
extern crate byteorder;
extern crate num_bigint;
extern crate num_traits;
extern crate rand;

use byteorder::{BigEndian, WriteBytesExt};
use bytes::ByteBuf;
use mio::tcp::TcpStream;
use mio::{TryRead, TryWrite};
use num_bigint::{BigUint, ToBigUint};
use num_traits::cast::ToPrimitive;
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

pub fn encrypt(msg: &String, method: &EncryptionMethod, secret: &u32) -> Vec<u8> {
    match method {
        EncryptionMethod::Cezar => return encrypt_cezar(msg, secret),
        EncryptionMethod::Xor => return encrypt_xor(msg, secret),
        _ => msg.clone().as_bytes().iter().cloned().collect(),
    }
}

pub fn decrypt(msg: Vec<u8>, method: &EncryptionMethod, secret: &u32) -> String {
    match method {
        EncryptionMethod::Cezar => return decrypt_cezar(msg, secret),
        EncryptionMethod::Xor => return decrypt_xor(msg, secret),
        _ => String::from_utf8(msg).unwrap(),
    }
}

fn encrypt_xor(msg: &String, secret: &u32) -> Vec<u8> {
    let mut secret_bytes = vec![];
    secret_bytes.write_u32::<BigEndian>(*secret).unwrap();
    let encrypted_msg_bytes: Vec<u8> = msg
        .as_bytes()
        .iter()
        .map(|byte| byte ^ secret_bytes.iter().last().unwrap())
        .collect();
    encrypted_msg_bytes
}

fn encrypt_cezar(msg: &String, secret: &u32) -> Vec<u8> {
    let encrypted_msg_bytes: Vec<u8> = msg
        .as_bytes()
        .iter()
        .map(|byte| byte.wrapping_add(*secret as u8))
        .collect();
    encrypted_msg_bytes
}

fn decrypt_cezar(msg: Vec<u8>, secret: &u32) -> String {
    let decrypted_msg_bytes: Vec<u8> = msg
        .iter()
        .map(|byte| byte.wrapping_sub(*secret as u8))
        .collect();
    String::from_utf8(decrypted_msg_bytes).unwrap()
}

fn decrypt_xor(msg: Vec<u8>, secret: &u32) -> String {
    let mut secret_bytes = vec![];
    secret_bytes.write_u32::<BigEndian>(*secret).unwrap();
    let encrypted_msg_bytes: Vec<u8> = msg
        .iter()
        .map(|byte| byte ^ secret_bytes.iter().last().unwrap())
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

    match socket.try_write(json_string.as_bytes()) {
        Ok(_) => return Ok(()),
        Err(e) => return Err(e.to_string()),
    }
}

pub fn generate_private_number() -> u32 {
    rand::random()
}

pub fn biguint(uint: &u32) -> BigUint {
    ToBigUint::to_biguint(uint).unwrap()
}

pub fn generate_public_number(p: u32, g: u32, a: u32) -> u32 {
    debug!("g: {}, a: {}", g, a);
    biguint(&g)
        .modpow(&biguint(&a), &biguint(&p))
        .to_u32()
        .unwrap()
}

pub fn find_secret(public: u32, private: u32, p: u32) -> u32 {
    debug!(
        "finding secret, public {}, private {}, p {}",
        public, private, p
    );
    biguint(&public)
        .modpow(&biguint(&private), &biguint(&p))
        .to_u32()
        .unwrap()
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

    let mut string_buf = String::new();
    data.read_to_string(&mut string_buf).unwrap();
    debug!("data before encoding msg: {:?}", string_buf);

    match json::parse(&string_buf) {
        Ok(j) => return Ok(j),
        Err(_) => return Err(String::from("Could not parse data into jason")),
    };
}

//
//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_xor_test() {
        let encrypted = encrypt(&String::from("msg"), &EncryptionMethod::Xor, &5);
        let decrypted = decrypt(encrypted, &EncryptionMethod::Xor, &5);

        assert_eq!("msg", decrypted);
    }

    #[test]
    fn encrypt_decrypt_cezar_test() {
        let encrypted = encrypt(&String::from("msg"), &EncryptionMethod::Cezar, &5);
        let decrypted = decrypt(encrypted, &EncryptionMethod::Cezar, &5);

        assert_eq!("msg", decrypted);
    }
}

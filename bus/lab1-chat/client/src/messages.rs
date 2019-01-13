use common::{EncryptionMethod, Message};
use json;

#[derive(Debug)]
pub struct SendParamReqMessage;

impl Message for SendParamReqMessage {
    fn create_json(&self) -> json::JsonValue {
        object!{
            "request" => "keys",
        }
    }

    fn get_msg_name(&self) -> String {
        return String::from("SendParamReqMessage");
    }
}

#[derive(Debug, Clone)]
pub struct SendEncryptionMethodMessage {
    method: EncryptionMethod,
}

impl SendEncryptionMethodMessage {
    pub fn new(method: EncryptionMethod) -> SendEncryptionMethodMessage {
        SendEncryptionMethodMessage { method: method }
    }
}

impl Message for SendEncryptionMethodMessage {
    fn create_json(&self) -> json::JsonValue {
        let method_str = match self.method {
            EncryptionMethod::None => String::from("none"),
            EncryptionMethod::Xor => String::from("xor"),
            EncryptionMethod::Cezar => String::from("cezar"),
        };
        object!{
            "encryption" => method_str,
        }
    }

    fn get_msg_name(&self) -> String {
        return String::from("SendEncryptionMethodMessage");
    }
}

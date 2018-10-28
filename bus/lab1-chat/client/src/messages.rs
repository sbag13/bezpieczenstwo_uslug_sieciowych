use common::Message;
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

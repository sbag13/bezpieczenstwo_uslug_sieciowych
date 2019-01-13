use common::Message;
use json;

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

    fn get_msg_name(&self) -> String {
        return String::from(format!("SendParamMessage({},{})", self.p, self.g));
    }
}

impl SendParamsMessage {
    pub fn new(p: u32, g: u32) -> SendParamsMessage {
        SendParamsMessage { p: p, g: g }
    }
}

//
//
// TESTS
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_json_test() {
        assert_eq!(
            SendParamsMessage::new(3, 4).create_json().dump(),
            r#"{"p":3,"g":4}"#
        );
    }
}

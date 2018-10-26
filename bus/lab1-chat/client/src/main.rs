extern crate mio;
#[macro_use]
extern crate json;
extern crate base64;
#[macro_use]
extern crate log;
extern crate env_logger;

use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, Handler, PollOpt, Token, TryWrite};
use std::net::SocketAddr;

const ADDRESS: &'static str = "127.0.0.1:12345";

fn main() {
    env_logger::init();

    let mut event_loop = EventLoop::new().unwrap();

    let address = ADDRESS.parse::<SocketAddr>().unwrap();

    let mut client_socket = ClientSocket {
        socket: TcpStream::connect(&address).unwrap(),
    };

    event_loop
        .register(
            &client_socket.socket,
            Token(0),
            EventSet::writable(),
            PollOpt::edge(),
        ).unwrap();

    match event_loop.run(&mut client_socket) {
        Ok(()) => info!("Event loop exited with success"),
        Err(err) => error!("Err: {}", err),
    }
}

struct ClientSocket {
    socket: TcpStream,
}

impl Handler for ClientSocket {
    type Timeout = usize;
    type Message = ();
    // TODO err, conn refused
    fn ready(&mut self, event_loop: &mut EventLoop<ClientSocket>, token: Token, events: EventSet) {
        debug!("Events: {:?}", events);

        self.send_param_request();
    }
}

impl ClientSocket {
    fn send_param_request(&mut self) {
        info!("Sending param request");
        let param_req_json = object!{
            "request" => "keys"
        };
        self.send_json(param_req_json);
    }

    fn send_json(&mut self, json: json::JsonValue) {
        let json_string = json.dump();
        debug!("param json string: {:?}", json_string);
        let string_base64 = base64::encode(json.dump().as_bytes());
        debug!("string in base64: {:?}", string_base64);
        self.socket.try_write(string_base64.as_bytes()).unwrap();
    }
}

extern crate base64;
extern crate bytes;
extern crate mio;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate json;

use bytes::ByteBuf;
use mio::tcp::{TcpListener, TcpStream};
use mio::{EventLoop, EventSet, Handler, PollOpt, Token, TryRead};
use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;

const ADDRESS: &'static str = "127.0.0.1:12345";
const SERVER_TOKEN: Token = Token(0);

fn main() {
    env_logger::init();

    let mut event_loop = EventLoop::new().unwrap();

    let address = ADDRESS.parse::<SocketAddr>().unwrap();

    let mut server = SocketServer {
        token_counter: 1,
        clients: HashMap::new(),
        socket: TcpListener::bind(&address).unwrap(),
    };

    event_loop
        .register(
            &server.socket,
            Token(0),
            EventSet::readable(),
            PollOpt::edge(),
        ).unwrap();

    info!(
        "Listening for incomming connections on: {:?}",
        address.port()
    );

    match event_loop.run(&mut server) {
        Ok(()) => info!("Event loop exited with success"),
        Err(err) => error!("Err: {}", err),
    }
}

struct SocketServer {
    socket: TcpListener,
    clients: HashMap<Token, SocketClient>,
    token_counter: usize,
}

impl Handler for SocketServer {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token, events: EventSet) {
        debug!("Events {:?} for token: {:?}", events, token);

        if events.is_hup() {
            // TODO the same for errors
            self.handle_hup(event_loop, token);
        } else if events.is_readable() {
            self.read(event_loop, token);
        }
    }
}

impl SocketServer {
    fn handle_hup(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token) {
        info!("Deregister client with token: {:?}", token);
        event_loop
            .deregister(&self.clients.get(&token).unwrap().socket)
            .unwrap();
        self.clients.remove(&token);
    }

    fn read(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token) {
        match token {
            SERVER_TOKEN => {
                self.handle_new_connection(event_loop);
            }
            token => {
                let mut client = self.clients.get_mut(&token).unwrap();
                client.read();
            }
        }
    }

    fn handle_new_connection(&mut self, event_loop: &mut EventLoop<SocketServer>) {
        let (client_socket, client_address) = match self.socket.accept() {
            Err(e) => {
                error!("Accept error: {}", e);
                return;
            }
            Ok(None) => unreachable!("Accept has returned 'None"),
            Ok(Some((sock, addr))) => {
                info!("{}", format!("Connection accepted from {:?}", addr));
                (sock, addr)
            }
        };

        let new_token = Token(self.token_counter);
        self.token_counter += 1;

        self.clients
            .insert(new_token, SocketClient::new(client_socket, client_address));
        event_loop
            .register(
                &self.clients[&new_token].socket,
                new_token,
                EventSet::all(), //TODO dont take all
                PollOpt::edge(),
            ).unwrap();
    }
}

#[derive(Debug, PartialEq)]
enum ClientState {
    Connected,
    ParamRequestReceived,
}

struct SocketClient {
    socket: TcpStream,
    address: SocketAddr,
    state: ClientState,
}

impl SocketClient {
    fn read(&mut self) {
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

    fn handle_data(&mut self, data: &mut ByteBuf) {
        match self.state {
            ClientState::Connected => self.handle_param_req(data),
            ClientState::ParamRequestReceived => self.handle_incoming_number(data),
        }
    }

    fn handle_incoming_number(&mut self, data: &mut ByteBuf) {
        let decoded_string = decode_to_string(data);
    }

    fn handle_param_req(&mut self, data: &mut ByteBuf) {
        if validate_param_req(&decode_to_string(data)) {
            info!("Param request received from: {:?}", self.address);
            self.state = ClientState::ParamRequestReceived;
        } else {
            unimplemented!()
        }
    }

    fn new(socket: TcpStream, addr: SocketAddr) -> SocketClient {
        SocketClient {
            socket: socket,
            address: addr,
            state: ClientState::Connected,
        }
    }
}

fn decode_to_string(data: &mut ByteBuf) -> String {
    let mut string_buf = String::new();
    data.read_to_string(&mut string_buf).unwrap();
    debug!("base64 string: {:?}", string_buf);

    let bytes_decoded = base64::decode(&string_buf).unwrap();
    debug!("decoded raw bytes: {:?}", bytes_decoded);

    let string_decoded = String::from(std::str::from_utf8(&bytes_decoded).unwrap());
    debug!("string decoded: {:?}", string_decoded);

    string_decoded
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

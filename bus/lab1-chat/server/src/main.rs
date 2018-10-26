extern crate bytes;
extern crate base64;
extern crate json;
extern crate mio;

use bytes::ByteBuf;
use mio::tcp::{TcpListener, TcpStream};
use mio::{EventLoop, EventSet, Handler, PollOpt, Token, TryRead};
use std::collections::HashMap;
use std::io::Read;
use std::net::SocketAddr;

const ADDRESS: &'static str = "127.0.0.1:12345";
const SERVER_TOKEN: Token = Token(0);

fn main() {
    // TODO logi

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

    match event_loop.run(&mut server) {
        Ok(()) => println!("Event loop exited with success"),
        Err(err) => println!("Err: {}", err),
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
        println!("Events {:?} for token: {:?}", events, token);

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
        println!("Deregister client with token: {:?}", token);
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
        let client_socket = match self.socket.accept() {
            Err(e) => {
                println!("Accept error: {}", e);
                return;
            }
            Ok(None) => unreachable!("Accept has returned 'None"),
            Ok(Some((sock, addr))) => sock,
        };

        let new_token = Token(self.token_counter);
        self.token_counter += 1;

        self.clients
            .insert(new_token, SocketClient::new(client_socket));
        event_loop
            .register(
                &self.clients[&new_token].socket,
                new_token,
                EventSet::all(),
                PollOpt::edge(), // TODO read about oneshot
            ).unwrap();
    }
}

struct SocketClient {
    socket: TcpStream,
}

impl SocketClient {
    fn read(&mut self) {
        loop {
            println!("Trying to read from socket {:?}", self.socket);
            let mut mut_buf = ByteBuf::mut_with_capacity(2048);
            match self.socket.try_read_buf(&mut mut_buf) {
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return;
                }
                Ok(None) =>
                // buff has no more bytes
                {
                    break;
                }
                Ok(Some(length)) => {
                    println!("data received from {:?}: \n{:?}", self.socket, mut_buf);
                    self.handle_data(&mut mut_buf.flip());
                    break;
                }
            }
        }
    }

    fn handle_data(&mut self, data: &mut ByteBuf) {
        let mut string_buf = String::new();
        data.read_to_string(&mut string_buf);
        let bytes_decoded = base64::decode(&string_buf).unwrap();
        println!("{:?}", bytes_decoded);
        let string_decoded = String::from(std::str::from_utf8(&bytes_decoded).unwrap());
        println!("string decoded: {:?}", string_decoded);
    }

    fn new(socket: TcpStream) -> SocketClient {
        SocketClient { socket: socket }
    }
}

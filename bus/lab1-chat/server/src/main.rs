extern crate json;
extern crate mio;

// use mio::{Handler, EventLoop, EventSet, Token, PollOpt};
use mio::tcp::{TcpListener, TcpStream};
use mio::*;
use std::collections::HashMap;
use std::net::SocketAddr;

fn main() {
    // TODO logi

    let mut event_loop = EventLoop::new().unwrap();

    let address = "0.0.0.0:12345".parse::<SocketAddr>().unwrap();
    let server_socket = TcpListener::bind(&address).unwrap();

    let mut server = SocketServer {
        token_counter: 1,
        clients: HashMap::new(),
        socket: server_socket,
    };

    match event_loop.run(&mut server) {
        Ok(()) => println!("Event loop exited with success"),
        Err(err) => println!("Err: {}", err),
    }

    event_loop
        .register(
            &server.socket,
            Token(0),
            EventSet::readable(),
            PollOpt::edge(),
        ).unwrap();
}

const SERVER_TOKEN: Token = Token(0);

struct SocketServer {
    socket: TcpListener,
    clients: HashMap<Token, SocketClient>,
    token_counter: usize,
}

impl Handler for SocketServer {
    type Timeout = usize;
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token, events: EventSet) {
        match token {
            SERVER_TOKEN => {
                let client_socket = match self.socket.accept() {
                    Err(e) => {
                        println!("Accept error: {}", e);
                        return;
                    }
                    Ok(None) => unreachable!("Accept has returned 'None"),
                    Ok(Some((sock, addr))) => sock,
                };

                self.token_counter += 1;
                let new_token = Token(self.token_counter);

                self.clients.insert(new_token, SocketClient::new(client_socket));
                event_loop
                    .register(
                        &self.clients[&new_token].socket,
                        new_token,
                        EventSet::readable(),
                        PollOpt::edge() | PollOpt::oneshot(), // TODO read about oneshot
                    ).unwrap();
            }
            token => {
                let mut client = self.clients.get_mut(&token).unwrap();
                client.read();
                event_loop
                    .reregister(
                        &client.socket,
                        token,
                        EventSet::readable(),
                        PollOpt::edge() | PollOpt::oneshot(),
                    ).unwrap();  //TODO maybe remove? without upgrades
            }
        }
    }
}

struct SocketClient {
    socket: TcpStream,
}

impl SocketClient {
    fn read(&mut self) {
        loop {
            let mut buf = [0; 2048];
            match self.socket.try_read(&mut buf) {
                Err(e) => {
                    println!("Error while reading socket: {:?}", e);
                    return;
                }
                Ok(None) =>
                // buff has no more bytes
                {
                    break
                }
                Ok(Some(length)) => {
                    // TODO parsing
                }
            }
        }
    }

    fn new(socket: TcpStream) -> SocketClient {
        SocketClient { socket: socket }
    }
}

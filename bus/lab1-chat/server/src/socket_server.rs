use mio::tcp::TcpListener;
use mio::{EventLoop, EventSet, Handler, PollOpt, Token};
use socket_client::SocketClient;
use std::collections::HashMap;

const SERVER_TOKEN: Token = Token(0);

pub struct SocketServer {
    pub socket: TcpListener,
    clients: HashMap<Token, SocketClient>,
    token_counter: usize,
}

impl Handler for SocketServer {
    type Timeout = usize;
    type Message = Token;

    fn notify(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token) {
        debug!("notified from {:?}", token);
        let client = self.clients.get_mut(&token).unwrap();
        SocketServer::reregister_writable(client, event_loop);
    }

    fn ready(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token, events: EventSet) {
        debug!("Events {:?} for token: {:?}", events, token);

        match token {
            SERVER_TOKEN => {
                // TODO the same for errors
                if events.is_hup() {
                    self.handle_hup(event_loop, token);
                } else if events.is_readable() {
                    self.handle_new_connection(event_loop);
                }
            }
            token => {
                let client = self.clients.get_mut(&token).unwrap();
                if events.is_readable() {
                    client.read();
                } else if events.is_writable() {
                    client.write();
                    if client.has_messages_to_sent() {
                        SocketServer::reregister_writable(&client, event_loop);
                    } else {
                        SocketServer::reregister_readable(&client, event_loop);
                    }
                }
            }
        }
    }
}

impl SocketServer {
    pub fn new(socket: TcpListener) -> SocketServer {
        SocketServer {
            socket: socket,
            clients: HashMap::new(),
            token_counter: 1,
        }
    }

    fn reregister_writable(client: &SocketClient, event_loop: &mut EventLoop<SocketServer>) {
        event_loop
            .reregister(
                &client.socket,
                client.token,
                EventSet::writable(),
                PollOpt::edge() | PollOpt::oneshot(),
            ).unwrap();
    }

    fn reregister_readable(client: &SocketClient, event_loop: &mut EventLoop<SocketServer>) {
        event_loop
            .reregister(
                &client.socket,
                client.token,
                EventSet::readable(),
                PollOpt::edge() | PollOpt::oneshot(),
            ).unwrap();
    }

    fn handle_hup(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token) {
        info!("Deregister client with token: {:?}", token);
        event_loop
            .deregister(&self.clients.get(&token).unwrap().socket)
            .unwrap();
        self.clients.remove(&token);
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

        self.clients.insert(
            new_token,
            SocketClient::new(
                new_token.clone(),
                client_socket,
                client_address,
                event_loop.channel(),
            ),
        );
        event_loop
            .register(
                &self.clients[&new_token].socket,
                new_token,
                EventSet::readable(),
                PollOpt::edge() | PollOpt::oneshot(),
            ).unwrap();
    }
}

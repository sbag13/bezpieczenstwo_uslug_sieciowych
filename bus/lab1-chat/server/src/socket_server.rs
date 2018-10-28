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
    type Message = ();

    fn ready(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token, events: EventSet) {
        debug!("Events {:?} for token: {:?}", events, token);

        if events.is_hup() {
            // TODO the same for errors
            // TODO distinguish server from client here
            self.handle_hup(event_loop, token);
        } else if events.is_readable() {
            self.read(event_loop, token);
        } else if events.is_writable() {
            self.clients.get_mut(&token).unwrap().write();
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
                EventSet::readable(),
                PollOpt::edge() | PollOpt::oneshot(),
            ).unwrap();
    }
}

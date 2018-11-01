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
    type Message = (Token, EventSet);

    fn notify(
        &mut self,
        event_loop: &mut EventLoop<SocketServer>,
        (token, event_set): (Token, EventSet),
    ) {
        debug!("notified from {:?}", token);
        let client = self.clients.get_mut(&token).unwrap();
        event_loop
            .reregister(
                &client.socket,
                client.token,
                event_set,
                PollOpt::edge() | PollOpt::oneshot(),
            ).unwrap();
    }

    fn ready(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token, events: EventSet) {
        match token {
            SERVER_TOKEN => {
                debug!("Events {:?} for token server socket", events);
                if events.is_hup() || events.is_error() {
                    //TODO server socket error
                    unimplemented!()
                } else if events.is_readable() {
                    self.handle_new_connection(event_loop);
                }
            }
            token => {
                if events.is_hup() || events.is_error() {
                    self.handle_client_err(event_loop, token);
                } else {
                    debug!(
                        "Events {:?} for token client socket with token: {:?}",
                        events, token
                    );
                    let client = self.clients.get_mut(&token).unwrap();
                    client.handle_event(event_loop, events);
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

    fn handle_client_err(&mut self, event_loop: &mut EventLoop<SocketServer>, token: Token) {
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

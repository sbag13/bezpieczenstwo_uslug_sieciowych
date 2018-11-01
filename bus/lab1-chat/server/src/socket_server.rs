use json;
use mio::tcp::TcpListener;
use mio::{EventLoop, EventSet, Handler, PollOpt, Token};
use socket_client::SocketClient;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

const SERVER_TOKEN: Token = Token(0);

pub struct SocketServer {
    pub socket: TcpListener,
    clients: HashMap<Token, SocketClient>,
    messages_to_broadcast: Rc<RefCell<VecDeque<(Token, json::JsonValue)>>>,
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
        debug!("notified for {:?}", token);

        if token == SERVER_TOKEN {
            while let Some(message) = self.messages_to_broadcast.borrow_mut().pop_front() {
                for item in self.clients.iter_mut() {
                    if *item.0 != message.0 {
                        item.1.push_normal_message_json(message.1.clone());
                        item.1.reregister(EventSet::writable());
                    }
                }
            }
        } else {
            let client = self.clients.get_mut(&token).unwrap();
            event_loop
                .reregister(
                    &client.socket,
                    client.token,
                    event_set,
                    PollOpt::edge() | PollOpt::oneshot(),
                ).unwrap();
        }
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
            messages_to_broadcast: Rc::new(RefCell::new(VecDeque::new())),
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
                self.messages_to_broadcast.clone(),
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

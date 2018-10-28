use common::{Message, generate_public_number, ClientState, read_json_from_socket};
use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, Handler, Token, PollOpt, Sender};
use messages::SendParamReqMessage;

pub struct ClientSocket {
    //TODO maybe move to common
    pub socket: TcpStream,
    state: ClientState,
    token: Token,
    event_loop_channel: Sender<EventSet>,
}

impl Handler for ClientSocket {
    type Timeout = usize;
    type Message = EventSet;
    // TODO err, conn refused
    fn ready(
        &mut self,
        _event_loop: &mut EventLoop<ClientSocket>,
        _token: Token,
        events: EventSet,
    ) {
        debug!("Events: {:?}", events);

        match self.state {
            ClientState::NotConnected => self.send_param_request(),
            ClientState::ParamReqSent => {
                if events.is_readable() {
                    self.read_params();
                }
            }
            ClientState::ParamsReceived => unimplemented!()
        }
    }

    fn notify(&mut self, event_loop: &mut EventLoop<ClientSocket>, events: EventSet) {
        debug!("notified with events: {:?}", events);
        event_loop.reregister(
            &self.socket,
            self.token,
            events,
            PollOpt::edge() | PollOpt::oneshot(),
        ).unwrap();
    }
}

impl ClientSocket {
    pub fn new(socket: TcpStream, event_loop_channel: Sender<EventSet>) -> ClientSocket {
        ClientSocket {
            socket: socket,
            state: ClientState::NotConnected,
            token: Token(0),
            event_loop_channel: event_loop_channel,
        }
    }

    // fn send_public_number(&mut self) {
    //     info!("Sendin public numerb to server");
    //     let client_public_number = generate_public_number();
    //     let number_json = object!{
    //         "a" => client_public_number
    //     };
    //     // TODO state
    // }

    // fn try_read_server_pub_number(&mut self) {
    //     unimplemented!()
    // }

    fn send_param_request(&mut self) {
        info!("Sending param request");
        SendParamReqMessage.send(&mut self.socket).unwrap(); // TODO wrap
        self.state = ClientState::ParamReqSent;
        self.event_loop_channel.send(EventSet::readable()).unwrap();  // TODO wrap
    }

    fn read_params(&mut self) {
        let json = read_json_from_socket(&mut self.socket).unwrap();    //TODO wrap
        debug!("received json: {:?}", json.dump());
        self.state = ClientState::ParamsReceived;
        self.event_loop_channel.send(EventSet::writable() | EventSet::readable()).unwrap();
    }
}


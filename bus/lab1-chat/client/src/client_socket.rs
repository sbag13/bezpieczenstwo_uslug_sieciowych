use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, Handler, Token};
use common::{ClientState, send_json_to_socket, generate_public_number};

pub struct ClientSocket {   //TODO maybe move to common
    pub socket: TcpStream,
    state: ClientState,
}

impl Handler for ClientSocket {
    type Timeout = usize;
    type Message = ();
    // TODO err, conn refused
    fn ready(&mut self, _event_loop: &mut EventLoop<ClientSocket>, _token: Token, events: EventSet) {
        debug!("Events: {:?}", events);

        match self.state {
            ClientState::NotConnected => self.send_param_request(),
            ClientState::ParamReqSent => {
                // if events.is_writable() {
                //     self.send_public_number();
                // } else if events.is_readable() {
                //     self.try_read_server_pub_number();
                // }
                ()
            }
        }
    }
}

impl ClientSocket {
    
    pub fn new(socket: TcpStream) -> ClientSocket {
        ClientSocket {
            socket: socket,
            state: ClientState::NotConnected,
        }
    }
    fn send_public_number(&mut self) {
        info!("Sendin public numerb to server");
        let client_public_number = generate_public_number();
        let number_json = object!{
            "a" => client_public_number
        };
        // TODO state
    }

    fn try_read_server_pub_number(&mut self) {
        unimplemented!()
    }

    fn send_param_request(&mut self) {
        info!("Sending param request");
        let param_req_json = object!{
            "request" => "keys"
        };
        send_json_to_socket(&mut self.socket, param_req_json); // TODO results
        self.state = ClientState::ParamReqSent;
    }
}
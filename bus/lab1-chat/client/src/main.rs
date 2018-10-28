extern crate mio;
#[macro_use]
extern crate json;
extern crate base64;
#[macro_use]
extern crate log;
extern crate common;
extern crate env_logger;

mod client_socket;
mod messages;

use client_socket::ClientSocket;
use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, PollOpt, Token};
use std::net::SocketAddr;

const ADDRESS: &'static str = "127.0.0.1:12345";

fn main() {
    env_logger::init();

    let mut event_loop = EventLoop::new().unwrap();

    let address = ADDRESS.parse::<SocketAddr>().unwrap();

    let mut client_socket =
        ClientSocket::new(TcpStream::connect(&address).unwrap(), event_loop.channel());

    event_loop
        .register(
            &client_socket.socket,
            Token(0),
            EventSet::writable(),
            PollOpt::edge() | PollOpt::oneshot(),
        ).unwrap();

    match event_loop.run(&mut client_socket) {
        Ok(()) => info!("Event loop exited with success"),
        Err(err) => error!("Err: {}", err),
    }
}

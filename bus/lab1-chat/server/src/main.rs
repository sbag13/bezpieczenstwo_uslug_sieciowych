extern crate base64;
extern crate bytes;
extern crate mio;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate json;
extern crate common;

mod messages;
mod socket_client;
mod socket_server;

use mio::tcp::TcpListener;
use mio::{EventLoop, EventSet, PollOpt, Token};
use socket_server::SocketServer;
use std::net::SocketAddr;

const ADDRESS: &'static str = "127.0.0.1:12345";

fn main() {
    env_logger::init();

    let mut event_loop = EventLoop::new().unwrap();

    let address = ADDRESS.parse::<SocketAddr>().unwrap();

    let mut server = SocketServer::new(TcpListener::bind(&address).unwrap());

    event_loop
        .register(&server.socket, Token(0), EventSet::all(), PollOpt::edge())
        .unwrap();

    info!(
        "Listening for incomming connections on: {:?}",
        address.port()
    );

    match event_loop.run(&mut server) {
        Ok(()) => info!("Event loop exited with success"),
        Err(err) => error!("Err: {}", err),
    }
}

// TODO
// two json quickly as one,/*  */ problem
// tests

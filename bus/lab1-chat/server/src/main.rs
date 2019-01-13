extern crate base64;
extern crate bytes;
extern crate mio;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate json;
extern crate common;
extern crate num_bigint;
extern crate primes;
extern crate rand;
extern crate clap;

mod messages;
mod parameters;
mod socket_client;
mod socket_server;

use mio::tcp::TcpListener;
use mio::{EventLoop, EventSet, PollOpt, Token};
use socket_server::SocketServer;
use std::net::SocketAddr;
use clap::{App, Arg, ArgMatches};

fn main() {
    env_logger::init();

    let args = get_args();
    let ip_address = args.value_of("address").unwrap_or("0.0.0.0");
    let port = args.value_of("port").unwrap_or("12345");
    let full_addr = String::from(ip_address.to_owned() + ":" + port);
    let address = full_addr.parse::<SocketAddr>().unwrap();

    debug!("server address: {:?}", address);

    let mut event_loop = EventLoop::new().unwrap();

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

pub fn get_args() -> ArgMatches<'static> {
    App::new("server tcp chat")
        .version("1.0")
        .author("Szymon Baginski <baginski.szymon@gmail.com>")
        .arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .value_name("ADDRESS")
                .help("Sets ipv4 address of server. Default is 0.0.0.0")
                .takes_value(true),
        ).arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("PORT")
                .help("Sets port for server. Default is 12345")
                .takes_value(true),
        ).get_matches()
}
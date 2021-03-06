extern crate mio;
#[macro_use]
extern crate json;
extern crate base64;
#[macro_use]
extern crate log;
extern crate clap;
extern crate common;
extern crate console;
extern crate env_logger;

mod cli_args;
mod client_socket;
mod messages;

use client_socket::ClientSocket;
use common::EncryptionMethod;
use console::Term;
use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, PollOpt, Token};
use std::net::SocketAddr;
use std::{io, thread};

fn main() {
    env_logger::init();
    let term = Term::stdout();

    let args = cli_args::get_args();

    let ip_address = args.value_of("address").unwrap_or("127.0.0.1");
    let port = args.value_of("port").unwrap_or("12345");
    let full_addr = String::from(ip_address.to_owned() + ":" + port);
    let address = full_addr.parse::<SocketAddr>().unwrap();

    let nickname = String::from(args.value_of("nickname").unwrap_or("anonymous"));

    let encryption = match args.value_of("encryption") {
        Some("XOR") => EncryptionMethod::Xor,
        Some("CEZAR") => EncryptionMethod::Cezar,
        Some("NONE") => EncryptionMethod::None,
        Some(_) => {
            println!("Unrecognized encryption method. set to none");
            EncryptionMethod::None
        }
        None => EncryptionMethod::None,
    };

    let mut event_loop = EventLoop::new().unwrap();

    let mut client_socket = ClientSocket::new(
        TcpStream::connect(&address).unwrap(),
        event_loop.channel(),
        nickname.clone(),
        encryption,
    );

    event_loop
        .register(
            &client_socket.socket,
            Token(0),
            EventSet::writable(),
            PollOpt::edge() | PollOpt::oneshot(),
        ).unwrap();

    let msg_channel = event_loop.channel();

    thread::spawn(move || match event_loop.run(&mut client_socket) {
        Ok(()) => info!("Event loop exited with success"),
        Err(err) => error!("Err: {}", err),
    });

    loop {
        let msg = get_line(&term, &nickname);
        if msg.is_ok() {
            match msg_channel.send((EventSet::writable(), Some(msg.unwrap()))) {
                Ok(_) => (),
                Err(_) => break,
            };
        } else {
            error!("{:?}", msg.unwrap());
            break;
        }
    }
}

fn get_line(term: &Term, nickname: &String) -> Result<String, std::io::Error> {
    let mut msg = String::new();
    io::stdin()
        .read_line(&mut msg)
        .expect("failed to read line");
    msg.pop();
    try!(term.move_cursor_up(1));
    try!(term.clear_line());
    try!(term.write_line(&format!("{} <<< {}", nickname, msg)));
    Ok(msg)
}

extern crate mio;
#[macro_use]
extern crate json;
extern crate base64;
#[macro_use]
extern crate log;
extern crate common;
extern crate env_logger;
extern crate console;

mod client_socket;
mod messages;

use client_socket::ClientSocket;
use mio::tcp::TcpStream;
use mio::{EventLoop, EventSet, PollOpt, Token};
use std::{io, thread};
use std::io::Write;
use std::net::SocketAddr;
use console::Term;

const ADDRESS: &'static str = "127.0.0.1:12345";

fn main() {
    env_logger::init();
    let term = Term::stdout();

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

    thread::spawn(move || match event_loop.run(&mut client_socket) {
        Ok(()) => info!("Event loop exited with success"),
        Err(err) => error!("Err: {}", err),
    });

    loop {
        term.write_line("### type text ### ");
        term.flush();
        let mut msg = String::new();
        io::stdin()
            .read_line(&mut msg)
            .expect("failed to read line");
        msg.pop();
        term.move_cursor_up(1);
        term.clear_line();
        term.move_cursor_up(1);
        term.clear_line();
        term.write_line(&format!("<<< {}", msg));
    }
}

// TODO maybe changestate fn

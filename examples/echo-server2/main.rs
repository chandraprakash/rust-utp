extern crate utp;
extern crate env_logger;

use utp::{UtpListener, UtpSocket};
use std::thread;

fn handle_client(mut s: UtpSocket) {
    let mut buf = [0; 1500];

    // Reply to a data packet with its src address, then end the connection
    match s.recv_from(&mut buf) {
        Ok((_, src)) => {
            println!("<= echo public address [{:?}]", src);
            let _ = s.send_to(src.to_string().as_bytes());
        }
        Err(e) => println!("{}", e)
    }
}

fn main() {
    // Start logger
    env_logger::init().unwrap();

    // Create a listener
    let addr = "0.0.0.0:5555";
    let listener = UtpListener::bind(addr).unwrap();
    println!("Started at {:?}", addr);
    for connection in listener.incoming() {
        // Spawn a new handler for each new connection
        match connection {
            Ok((socket, _src)) => { thread::spawn(move || { handle_client(socket) }); },
            _ => ()
        }
    }
}

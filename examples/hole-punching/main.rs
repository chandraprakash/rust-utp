//! Implementation of a simple uTP client and server.
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate utp;

use std::process;
use std::thread;
use std::str::FromStr;

// A little macro to make it easier to unwrap return values or halt the program
macro_rules! iotry {
    ($e:expr) => (match $e { Ok(v) => v, Err(e) => panic!("{}", e), })
}

fn usage() -> ! {
    println!("Usage: utp <outgoing_port> <address> <port>");
    process::exit(1);
}

fn main() {
    use utp::UtpStream;
    use std::io::{stdin, stdout, stderr, Read, Write};

    // Start logging
    env_logger::init().unwrap();

    // Fetch arguments
    let mut args = std::env::args();

    // Skip program name
    args.next();

    // Parse outgoing port
    let outgoing_port: u16 = match args.next() {
        Some(p) => iotry!(u16::from_str(&p)),
        _ => usage(),
    };

    // Parse the address argument or use a default if none is provided
    let dst_addr = match (args.next(), args.next()) {
        (Some(ip), Some(port)) => format!("{}:{}", ip, port),
        _ => usage(),
    };
    let dst_addr: &str = &dst_addr;
    let my_addr = format!("{}:{}", "0.0.0.0", outgoing_port);
    //let my_addr: &str = &my_addr;


//    loop {
        let listen_handler = thread::Builder::new().name("listen thread".to_string())
                                              .spawn(move || {
            // Create a listening stream
            let my_addr_clone = my_addr.clone();
            let my_addr_clone: &str = &my_addr_clone;
            let mut stream = iotry!(UtpStream::bind_with_reuse_address(my_addr_clone));
            let _ = writeln!(&mut stderr(), "Serving on {}", my_addr_clone);
        }).unwrap();

        // Create a stream and try to connect to the remote address
        let _ = writeln!(&mut stderr(), "trying to connected to {:?}; outgoing port {:?}", dst_addr, outgoing_port);
        let mut stream = iotry!(UtpStream::connect_with_reuse_address(dst_addr, outgoing_port));
        let _ = writeln!(&mut stderr(), "connected to {}", dst_addr);
        // TODO try private addresses as well!! in a separate thread
//    }

        // Explicitly close the stream.
        let _ = writeln!(&mut stderr(), "closing ....");
        iotry!(stream.close());

}

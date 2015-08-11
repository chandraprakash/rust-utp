//! Implementation of a simple uTP client and server.
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate utp;
extern crate net2;

use std::process;
use net2::UdpBuilder;
use std::str::FromStr;
use std::net::SocketAddr;

// A little macro to make it easier to unwrap return values or halt the program
macro_rules! iotry {
    ($e:expr) => (match $e { Ok(v) => v, Err(e) => panic!("{}", e), })
}

fn usage() -> ! {
    println!("Usage: utp [-s|-c] <own_port> <address> <port>");
    process::exit(1);
}

pub fn array_as_vector(arr: &[u8]) -> Vec<u8> {
  let mut vector = Vec::new();
  for i in arr.iter() {
    vector.push(*i);
  }
  vector
}

fn main() {
    use utp::UtpStream;
    use std::io::{stdin, stdout, stderr, Read, Write};

    // This example may run in either server or client mode.
    // Using an enum tends to make the code cleaner and easier to read.
    enum Mode {Server, Client}

    // Start logging
    env_logger::init().unwrap();

    // Fetch arguments
    let mut args = std::env::args();

    // Skip program name
    args.next();

    // Parse the mode argument
    let mode: Mode = match args.next() {
        Some(ref s) if s == "-s" => Mode::Server,
        Some(ref s) if s == "-c" => Mode::Client,
        _ => usage(),
    };

    // Parse the address argument or use a default if none is provided
    let addr = match (args.next(), args.next()) {
        (None, None) => "127.0.0.1:5555".to_owned(),
        (Some(ip), Some(port)) => format!("{}:{}", ip, port),
        _ => usage(),
    };
    let addr: &str = &addr;


    // get peer's public endpoint
    let udp_builder = iotry!(UdpBuilder::new_v4());
    let _ = iotry!(udp_builder.reuse_address(true));
    let udp_socket = iotry!(udp_builder.bind("0.0.0.0:0"));

    let mut peer_addr: Option<SocketAddr> = None;
    while true {
        iotry!(udp_socket.send_to(b"hello", &addr));
        println!("send_to(b\"hello\" to  {:?})", addr);
        let mut buf = [0; 100];
        let (amt, src) = iotry!(udp_socket.recv_from(&mut buf));
        let buf = &mut buf[..amt];
        let public_endpoint = String::from_utf8(array_as_vector(buf)).unwrap();

        println!("response: {:?}", public_endpoint);

        if public_endpoint == "retry" {
            continue;
        }


        match SocketAddr::from_str(&public_endpoint) {
            Ok(addr) => {
                peer_addr = Some(addr);
                println!("got peer's public endpoint {:?}", public_endpoint);
                break;
            },
            _ => { continue; }
        };
    }

    let local_addr = iotry!(udp_socket.local_addr());
    let peer_addr = peer_addr.unwrap();

    // get peer address !  // FIXME
    //let dst_addr = format!("{}:{}", "127.0.0.1", "5666"); // FIXME
    //let dst_addr2: &str = &public_endpoint;


    match mode {
        Mode::Server => {
            // Create a listening stream
            let mut stream = iotry!(UtpStream::bind_with_reuse_address(local_addr));
            let mut writer = stdout();
            let _ = writeln!(&mut stderr(), "Serving on {}", local_addr);

            // Create a reasonably sized buffer
            let mut payload = vec![0; 1024 * 1024];

            // Wait for a new connection and print the received data to stdout.
            // Reading and printing chunks like this feels more interactive than trying to read
            // everything with `read_to_end` and avoids resizing the buffer multiple times.
            loop {
                match stream.read(&mut payload) {
                    Ok(0) => break,
                    Ok(read) => iotry!(writer.write(&payload[..read])),
                    Err(e) => panic!("{}", e)
                };
            }
        }
        Mode::Client => {
            // Create a stream and try to connect to the remote address
            let mut stream = iotry!(UtpStream::connect_with_reuse_address(peer_addr,
                                                                          local_addr.port()));
            let mut reader = stdin();

            // Create a reasonably sized buffer
            let mut payload = vec![0; 1024 * 1024];

            // Read from stdin and send it to the remote server.
            // Once again, reading and sending small chunks like this avoids having to read the
            // entire input (which may be endless!) before starting to send, unlike what would
            // happen if we were to use `read_to_end` on `reader`.
            loop {
                match reader.read(&mut payload) {
                    Ok(0) => break,
                    Ok(read) => iotry!(stream.write(&payload[..read])),
                    Err(e) => {
                        iotry!(stream.close());
                        panic!("{:?}", e);
                    }
                };
            }

            // Explicitly close the stream.
            iotry!(stream.close());
        }
    }
}

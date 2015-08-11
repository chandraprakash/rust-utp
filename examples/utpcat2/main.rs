//! Implementation of a simple uTP client and server.
#![feature(duration, socket_timeout)]

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate utp;
extern crate net2;

use std::process;
use net2::UdpBuilder;
use std::str::FromStr;
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;


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
        (None, None) => "127.0.0.1:5556".to_owned(),
        (Some(ip), Some(port)) => format!("{}:{}", ip, port),
        _ => usage(),
    };
    let rendezvous_server_addr: &str = &addr;

    let mut peer_addr: Option<SocketAddr> = None;
    let mut local_addr: Option<SocketAddr> = None;
    {  // get peer's public endpoint (peer_addr) and own outgoing port (local_addr)

        let udp_builder = iotry!(UdpBuilder::new_v4());
        let _ = iotry!(udp_builder.reuse_address(true));
        let udp_socket = iotry!(udp_builder.bind("0.0.0.0:0"));

        while true {
            thread::sleep_ms(1000);
            iotry!(udp_socket.send_to(b"hello", &rendezvous_server_addr));
            println!("send_to(b\"hello\" to  {:?})", rendezvous_server_addr);
            let mut buf = [0; 100];
            let (amt, src) = iotry!(udp_socket.recv_from(&mut buf));
            let buf = &mut buf[..amt];
            let public_endpoint = String::from_utf8(array_as_vector(buf)).unwrap();

            println!("response: {:?}", public_endpoint);
            let x = iotry!(udp_socket.local_addr());
            local_addr = Some(x);
            println!("local_addr: {:?}", local_addr);


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
    }


    let local_addr = local_addr.unwrap();
    let peer_addr = peer_addr.unwrap();

    // Send data via same outgoing port to peer address until you receive data form it
    {
        let udp_builder = iotry!(UdpBuilder::new_v4());
        let _ = iotry!(udp_builder.reuse_address(true));
        let udp_socket = iotry!(udp_builder.bind(local_addr));

        while true {
            thread::sleep_ms(100);
            iotry!(udp_socket.send_to(b"punch", peer_addr));
            println!("sent \"punch\" to  {:?})", peer_addr);
            let mut buf = [0; 10];

            udp_socket.set_read_timeout(Some(Duration::new(1, 0)));
            match udp_socket.recv_from(&mut buf) {
                Ok(_) => {
                    //let buf = &mut buf[..amt];
                    //let response = String::from_utf8(array_as_vector(buf)).unwrap();
                    println!("Got message back from {:?}", peer_addr);
                    break;
                },
                _ => {
                    println!("Send again to {:?}", peer_addr);
                },
            }
        }
    }

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
            println!("connect local_addr: {:?} with reuse_address to  {:?}", local_addr, peer_addr);
            let mut stream = iotry!(UtpStream::connect_with_reuse_address(peer_addr,
                                                                          local_addr.port()));
            println!("connected from local_addr: {:?} to  {:?}", local_addr, peer_addr);
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

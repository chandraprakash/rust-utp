extern crate env_logger;
extern crate net2;

use net2::UdpBuilder;
use std::net::SocketAddr;

// A little macro to make it easier to unwrap return values or halt the program
macro_rules! iotry {
    ($e:expr) => (match $e { Ok(v) => v, Err(e) => panic!("{}", e), })
}

pub fn array_as_vector(arr: &[u8]) -> Vec<u8> {
  let mut vector = Vec::new();
  for i in arr.iter() {
    vector.push(*i);
  }
  vector
}

fn main() {
    // Start logger
    env_logger::init().unwrap();

    let udp_builder = iotry!(UdpBuilder::new_v4());
    let _ = iotry!(udp_builder.reuse_address(true));

    let socket = iotry!(udp_builder.bind("0.0.0.0:5555"));
    let local_addr = iotry!(socket.local_addr());
    println!("Started at {:?}", local_addr);

    let mut endpoints = vec![];

    while true {
        let mut buf = [0; 10];
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let buf = &mut buf[..amt];
                let msg = String::from_utf8(array_as_vector(buf)).unwrap();
                println!("message : [{:?}]", msg);
                println!("echo public address [{:?}]", src);

                match endpoints.len() {
                    0 => {
                        endpoints.push(src.clone());
                        // send "retry" try again to get b's endpoint
                        println!("send \"retry\" to {:?}; try again to get b's endpoint ", src);
                        let _ = socket.send_to("retry".as_bytes(), &src);
                    },
                    1 => {
                        if (endpoints[0] == src) {
                            // send "retry" try again to get b's endpoint
                            println!("send \"retry\" to {:?}; try again to get b's endpoint ", src);
                            let _ = socket.send_to("retry".as_bytes(), &src);
                        } else {
                            endpoints.push(src);
                            println!("send a:{:?} to b{:?} ", endpoints[0], src);
                            let _ = socket.send_to(endpoints[0].to_string().as_bytes(), &src);
                        }
                    },
                    2 => {
                        // send counterpart if src is present
                        if endpoints.iter().find(|&x| *x == src).is_some() {
                            let pos = endpoints.iter().position(|x| *x != src).unwrap();
                            println!("send a:{:?} to b{:?} ", endpoints[pos], src);
                            let _ = socket.send_to(endpoints[pos].to_string().as_bytes(), &src);
                            continue;
                        }

                        // or else empty vector; add new entry and send retry
                        endpoints.clear();
                        endpoints.push(src.clone());
                        // send "retry" try again to get b's endpoint
                        println!("clear !! send \"retry\" to {:?}; try again to get b's endpoint ", src);
                        let _ = socket.send_to("retry".as_bytes(), &src);
                    },
                    _ => panic!("{Something is wrong !!!}"),
                }

                let _ = socket.send_to(src.to_string().as_bytes(), &src);
            },
            Err(e) => println!("{}", e)
        }
    }
}

extern crate env_logger;
extern crate net2;

use net2::UdpBuilder;


// A little macro to make it easier to unwrap return values or halt the program
macro_rules! iotry {
    ($e:expr) => (match $e { Ok(v) => v, Err(e) => panic!("{}", e), })
}

fn main() {
    // Start logger
    env_logger::init().unwrap();

    let udp_builder = iotry!(UdpBuilder::new_v4());
    let _ = iotry!(udp_builder.reuse_address(true));

    let socket = iotry!(udp_builder.bind("0.0.0.0:5555"));
    let local_addr = iotry!(socket.local_addr());
    println!("Started at {:?}", local_addr);

    while true {
        let mut buf = [0; 10];
        match socket.recv_from(&mut buf) {
            Ok((_, src)) => {
                println!("<= echo public address [{:?}]", src);
                let _ = socket.send_to(src.to_string().as_bytes(), &src);
            },
            Err(e) => println!("{}", e)
        }
    }
}

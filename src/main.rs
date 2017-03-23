use std::net::{TcpListener, TcpStream};
use std::thread;
// use std::io::Read;
// use std::str;

mod socks5;
use socks5::socks5::TCPRelay;


fn handle(stream: TcpStream) {
    let mut socks5 = TCPRelay::new(stream, "1234556".to_string());
    socks5.serve();
    // let mut client_buffer = [0u8; 1024];
    // loop {
    //     stream.read(&mut client_buffer)
    //     match stream.read(&mut client_buffer) {
    //         Ok(n) => {
    //             if n == 0 {
    //                 return;
    //             } else {
    //                 println!("Read {}", str::from_utf8(&client_buffer).unwrap());
    //             }
    //         }
    //         Err(error) => {
    //             println!("{}", error.to_string());
    //         }
    //     };
    // }
}

fn main() {
    let listen = TcpListener::bind(("127.0.0.1", 1080)).unwrap();
    for stream in listen.incoming() {
        let s = stream.unwrap();
        thread::spawn(move || handle(s));
        // handler.join();
    }
}
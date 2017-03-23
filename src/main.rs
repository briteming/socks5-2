use std::net::{TcpListener, TcpStream};
use std::thread;

mod socks5;
use socks5::socks5::TCPRelay;

fn handle(stream: TcpStream) {
    let mut socks5 = TCPRelay::new(stream, "1234556".to_string());
    socks5.serve();
}

fn main() {
    let listen = TcpListener::bind(("127.0.0.1", 1080)).unwrap();
    for stream in listen.incoming() {
        let s = stream.unwrap();
        thread::spawn(move || handle(s));
    }
}

#![allow(dead_code)]

// Package socks5 implements socks5 proxy protocol.

use std::net::{TcpStream, Shutdown};
use std::io::{Read, Write};
use std::io;
use std::str;
use std::thread;
use std::time;

use socks5::address;

const SOCKSV5: u8 = 0x05;
const DEBUG: bool = false;

const CONNECT: u8 = 0x01;
const BIND: u8 = 0x02;
const UDP_ASSOCIATE: u8 = 0x03;

// TCPRelay as a socks5 server and mika client.
pub struct TCPRelay {
    conn: TcpStream,
    // cipher   *mika.Crypto
    ss_server: String,
    closed: bool,
}

impl TCPRelay {
    // TCPRelay::new creates a new Socks5 TCPRelay.
    pub fn new(conn: TcpStream, mika_server: String) -> TCPRelay {
        TCPRelay {
            conn: conn,
            ss_server: mika_server,
            closed: false,
        }
    }

    // serve handles connection between socks5 client and remote addr.
    pub fn serve(&mut self) {
        // if !s.closed {
        // s.conn.Close()
        // }
        self.hand_shake();

        // get cmd and address
        let (cmd, addr) = self.parse_request().unwrap();
        self.reply();

        match cmd {
            CONNECT => {;
                self.connect(addr);
            }
            UDP_ASSOCIATE => self.udp_associate(),
            BIND => {}
            _ => {}
        }

        println!("serve stopped");
        // let _ = self.conn.shutdown(Shutdown::Both);
    }

    // version identifier/method selection message
    // +----+----------+----------+
    // |VER | NMETHODS | METHODS  |
    // +----+----------+----------+
    // | 1  |    1     | 1 to 255 |
    // +----+----------+----------+
    // reply:
    // +----+--------+
    // |VER | METHOD |
    // +----+--------+
    // |  1 |   1    |
    // +----+--------+
    // hand_shake dail hand_shake between socks5 client and socks5 server.
    fn hand_shake(&mut self) {
        let mut raw = [0u8; 257];
        let _ = self.conn.read_exact(&mut raw[0..2]);
        // get socks version
        let ver = raw[0];
        if DEBUG {
            println!("Socks version {}", ver);
        }

        if ver != SOCKSV5 {
            println!("Error version {}", ver);
        }

        // read all method identifier octets
        let nmethods: usize = raw[1] as usize;
        if DEBUG {
            println!("Socks method {}", nmethods);
        }

        let _ = self.conn.read_exact(&mut raw[2..2 + nmethods]);

        // reply to socks5 client
        let _ = self.conn.write(&[SOCKSV5, 0x00]);
    }

    // The SOCKS request is formed as follows:
    //         +----+-----+-------+------+----------+----------+
    //         |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
    //         +----+-----+-------+------+----------+----------+
    //         | 1  |  1  | X’00’ |  1   | Variable |    2     |
    //         +----+-----+-------+------+----------+----------+
    // Where:
    //           o  VER    protocol version: X’05’
    //           o  CMD
    //              o  CONNECT X’01’
    //              o  BIND X’02’
    //              o  UDP ASSOCIATE X’03’
    //           o  RSV    RESERVED
    //           o  ATYP   address type of following address
    //              o  IP V4 address: X’01’
    //              o  DOMAINNAME: X’03’
    //              o  IP V6 address: X’04’
    //           o  DST.ADDR       desired destination address
    //           o  DST.PORT desired destination port in network octet order

    // get_cmd gets the cmd requested by socks5 client.
    fn get_cmd(&mut self) -> u8 {
        let mut raw = [0u8; 3];
        let _ = self.conn.read_exact(&mut raw);

        // get socks version
        let ver = raw[0];
        if ver != SOCKSV5 {
            println!("Error version {}", ver);
        }
        return raw[1];
    }

    // parse_request parses socks5 client request.
    fn parse_request(&mut self) -> io::Result<(u8, address::Address)> {
        let cmd = self.get_cmd();

        if DEBUG {
            println!("Cmd {}", cmd);
        }

        // check cmd type
        match cmd {
            CONNECT | BIND | UDP_ASSOCIATE => {}
            _ => {
                println!("unknow cmd type");
                return Err(io::Error::new(io::ErrorKind::Other, "unsupported address type"));
            }
        }

        let addr = address::get_address(&mut self.conn).unwrap();
        if DEBUG {
            println!("{:?}", addr);
        }
        Ok((cmd, addr))
    }

    // returns a reply formed as follows:
    //         +----+-----+-------+------+----------+----------+
    //         |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
    //         +----+-----+-------+------+----------+----------+
    //         | 1  |  1  | X’00’ |  1   | Variable |    2     |
    //         +----+-----+-------+------+----------+----------+
    // Where:
    //           o  VER    protocol version: X’05’
    //           o  REP    Reply field:
    //              o  X’00’ succeeded
    //              o  X’01’ general SOCKS server failure
    //              o  X’02’ connection not allowed by ruleset
    //              o  X’03’ Network unreachable
    //              o  X’04’ Host unreachable
    //              o  X’05’ Connection refused
    //              o  X’06’ TTL expired
    //              o  X’07’ Command not supported
    //              o  X’08’ Address type not supported
    //              o  X’09’ to X’FF’ unassigned
    //           o  RSV    RESERVED
    //           o  ATYP   address type of following address
    //              o  IP V4 address: X’01’
    //              o  DOMAINNAME: X’03’
    //              o  IP V6 address: X’04’
    //           o  BND.ADDR       server bound address
    //           o  BND.PORT       server bound port in network octet order
    fn reply(&mut self) {
        println!("Reply socks5 client");
        let _ = self.conn.write(&[SOCKSV5,
                                  0x00,
                                  0x00,
                                  address::IPV4_ADDR,
                                  0x00,
                                  0x00,
                                  0x00,
                                  0x00,
                                  0x10,
                                  0x10]);
    }

    // connect handles CONNECT cmd
    // Here is a bit magic. It acts as a mika client that redirects conntion to mika server.
    fn connect(&mut self, addr: address::Address) {
        // TODO Dail("tcp", rawAdd) would be more reasonable.
        // mikaConn, err := mika.DailWithraw_addr("kcp", s.ssServer, raw_addr, s.cipher);
        // if err != nil {
        // 	return;
        // }

        let mut remote = TcpStream::connect(addr).unwrap();
        let mut remote_copy = remote.try_clone().unwrap();
        let mut client_copy = self.conn.try_clone().unwrap();
        thread::spawn(move || {
            let mut client_buffer = [0u8; 4096];
            loop {
                match client_copy.read(&mut client_buffer) {
                    Ok(n) => {
                        remote_copy.write(&client_buffer[0..n]).unwrap();
                        // println!("read client write to client");
                    }
                    Err(error) => {
                        println!("{}", error.to_string());
                        return;
                    }
                }
            }
        });
        let mut client_buffer = [0u8; 4096];
        loop {
            remote.set_read_timeout(Some(time::Duration::from_secs(10)));
            match remote.read(&mut client_buffer) {
                Ok(n) => {
                    self.conn.write(&client_buffer[0..n]).unwrap();
                    self.conn.set_write_timeout(Some(time::Duration::from_secs(10)));
                    // println!("read remote write to client");
                }
                Err(error) => {
                    println!("{}", error.to_string());
                    return;
                }
            }

        }
        // self.closed = true;
    }

    // udp_associate handles UDP_ASSOCIATE cmd
    fn udp_associate(&mut self) {
        let _ = self.conn.write(&[SOCKSV5,
                                  0x00,
                                  0x00,
                                  address::IPV4_ADDR,
                                  0x00,
                                  0x00,
                                  0x00,
                                  0x00,
                                  0x04,
                                  0x38]);
    }
}

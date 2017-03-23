#![allow(dead_code)]

// Package socks5 implements socks5 proxy protocol.

use std::net::{TcpStream, Shutdown};
use std::io::Read;
use std::io::Write;

use socks5::address;

const SOCKSV5: u8 = 0x05;
const DEBUG: bool = true;

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

        self.parse_request();
        // let cmd, raw_addr, addr := self.parse_request();
        // // if err != nil {
        // // 	utils.Errorf("Parse request error %v\n", err)
        // // 	return
        // // }

        // // utils.Infof("Proxy connection to %s\n", string(addr))
        // self.reply();

        // match cmd {
        //  CONNECT=>{
        //     self.connect(raw_addr);
        // },
        // UDP_ASSOCIATE =>
        // 	self.udpAssociate(),
        //  BIND =>
        //  // error
        // }
        let _ = self.conn.shutdown(Shutdown::Both);
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
            // return fmt.Errorf("error socks version %d", ver)
        }

        // read all method identifier octets
        let nmethods: usize = raw[1] as usize;
        if DEBUG {
            println!("Socks method {}", nmethods);
        }

        let _ = self.conn.read_exact(&mut raw[2..2 + nmethods]);

        // reply to socks5 client
        let resp: [u8; 2] = [SOCKSV5, 0x00];
        let _ = self.conn.write(&resp);
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
        // if err != nil {
        // 	return
        // }

        // get socks version
        let ver = raw[0];
        if ver != SOCKSV5 {
            println!("Error version {}", ver);
            // return 0, fmt.Errorf("error socks version %d", ver)
        }
        return raw[1];
    }

    // parse_request parses socks5 client request.
    fn parse_request(&mut self) {
        let cmd = self.get_cmd();

        println!("Cmd {}", cmd);

        // check cmd type
        match cmd {
            CONNECT | BIND | UDP_ASSOCIATE => {}
            _ => {
                println!("unknow cmd type");
                return;
            }
        }

        let _ = address::get_address(&mut self.conn);
        // let raw_addr, addr, err = utils.GetAddress(s.conn)
        // if err != nil {
        // 	return;
        // }
    }

    // // returns a reply formed as follows:
    // //         +----+-----+-------+------+----------+----------+
    // //         |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
    // //         +----+-----+-------+------+----------+----------+
    // //         | 1  |  1  | X’00’ |  1   | Variable |    2     |
    // //         +----+-----+-------+------+----------+----------+
    // // Where:
    // //           o  VER    protocol version: X’05’
    // //           o  REP    Reply field:
    // //              o  X’00’ succeeded
    // //              o  X’01’ general SOCKS server failure
    // //              o  X’02’ connection not allowed by ruleset
    // //              o  X’03’ Network unreachable
    // //              o  X’04’ Host unreachable
    // //              o  X’05’ Connection refused
    // //              o  X’06’ TTL expired
    // //              o  X’07’ Command not supported
    // //              o  X’08’ Address type not supported
    // //              o  X’09’ to X’FF’ unassigned
    // //           o  RSV    RESERVED
    // //           o  ATYP   address type of following address
    // //              o  IP V4 address: X’01’
    // //              o  DOMAINNAME: X’03’
    // //              o  IP V6 address: X’04’
    // //           o  BND.ADDR       server bound address
    // //           o  BND.PORT       server bound port in network octet order
    // fn reply(&self) {
    //     let resp:[u8;10] = [socksv5, 0x00, 0x00, utils.IPV4_ADDR, 0x00, 0x00, 0x00, 0x00, 0x10, 0x10];
    // 	s.conn.Write(resp);
    // }

    // // connect handles CONNECT cmd
    // // Here is a bit magic. It acts as a mika client that redirects conntion to mika server.
    // fn connect(&self, raw_addr: &[u8]) -> error {
    // 	// TODO Dail("tcp", rawAdd) would be more reasonable.
    // 	// mikaConn, err := mika.DailWithraw_addr("kcp", s.ssServer, raw_addr, s.cipher);
    // 	// if err != nil {
    // 	// 	return;
    // 	// }

    // 	// defer func() {
    // 	// 	if !s.closed {
    // 	// 		err := mikaConn.Close()
    // 	// 		utils.Errorf("Close connection error %v\n", err)
    // 	// 	}
    // 	// }()

    // 	// go protocols.Pipe(s.conn, mikaConn)
    // 	// protocols.Pipe(mikaConn, s.conn)
    // 	self.closed = true;
    // }

    // // udpAssociate handles UDP_ASSOCIATE cmd
    // fn udpAssociate(&self) {
    //         let resp:[u8;10] = [socksv5, 0x00, 0x00, utils.IPV4_ADDR, 0x00, 0x00, 0x00, 0x00, 0x04, 0x38];
    // 	self.conn.Write(resp);
    // }
}
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str;


const DEBUG: bool = true;

pub const IPV4_ADDR: u8 = 0x1;
pub const DOMAIN_ADDR: u8 = 0x3;
pub const IPV6_ADDR: u8 = 0x4;

pub const IPV4_LEN: usize = 4;
pub const IPV6_LEN: usize = 16;
pub const PORT_LEN: usize = 2;

// +------+----------+----------+
// | ATYP | DST.ADDR | DST.PORT |
// +------+----------+----------+
// |  1   | Variable |    2     |
// +------+----------+----------+
// o  ATYP    address type of following addresses:
// 		o  IP V4 address: X’01’
// 		o  DOMAINNAME: X’03’
// 		o  IP V6 address: X’04’
// o  DST.ADDR		desired destination address
// o  DST.PORT		desired destination port in network octet
// In an address field (DST.ADDR, BND.ADDR), the ATYP field specifies
//    the type of address contained within the field:
//			o  X’01’
//    the address is a version-4 IP address, with a length of 4 octets
// 			o X’03’
//    the address field contains a fully-qualified domain name.  The first
//    octet of the address field contains the number of octets of name that
//    follow, there is no terminating NUL octet.
//			o  X’04’
//    the address is a version-6 IP address, with a length of 16 octets.

// (raw []byte, addr string, err error)
pub fn get_address<T: io::Read>(r: &mut T) -> io::Result<SocketAddr> {
    let mut raw = [0u8; 1];

    let _ = r.read_exact(&mut raw[0..1]);

    let atyp = raw[0];
    if DEBUG {
        println!("ATYP {}", atyp);
    }

    let mut raw_addr_len = PORT_LEN;
    match atyp {
        IPV4_ADDR => {
            raw_addr_len = raw_addr_len + IPV4_LEN;
        }
        DOMAIN_ADDR => {
            let _ = r.read_exact(&mut raw[0..1]);
            raw_addr_len = raw_addr_len + raw[0] as usize;
        }
        IPV6_ADDR => {
            raw_addr_len = raw_addr_len + IPV6_LEN;
        }
        _ => {
            println!("unsupported address type");
            return Err(io::Error::new(io::ErrorKind::Other, "unsupported address type"));
        }
    }

    let mut raw_addr = [0u8; 260];
    let _ = r.read_exact(&mut raw_addr[0..raw_addr_len]);

    let port = raw_addr[raw_addr_len - PORT_LEN] as u16 * 256 + raw_addr[raw_addr_len - 1] as u16;
    println!("Port {}", port);
    match atyp {
        IPV4_ADDR => {
            let ip = Ipv4Addr::new(raw_addr[0], raw_addr[1], raw_addr[2], raw_addr[3]);
            println!("{}", ip);
            return Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)));
        }
        DOMAIN_ADDR => {
            let host = str::from_utf8(&raw_addr[0..raw_addr_len - PORT_LEN]).unwrap().to_string();
            println!("DOMAIN_ADDR {}", host);
        }
        IPV6_ADDR => {
            let ip = Ipv6Addr::new(raw_addr[0] as u16 * 256 + raw_addr[1] as u16,
                                   raw_addr[2] as u16 * 256 + raw_addr[3] as u16,
                                   raw_addr[4] as u16 * 256 + raw_addr[5] as u16,
                                   raw_addr[6] as u16 * 256 + raw_addr[7] as u16,
                                   raw_addr[8] as u16 * 256 + raw_addr[9] as u16,
                                   raw_addr[10] as u16 * 256 + raw_addr[11] as u16,
                                   raw_addr[12] as u16 * 256 + raw_addr[13] as u16,
                                   raw_addr[14] as u16 * 256 + raw_addr[15] as u16);
            println!("Ipv6 {}", ip);
            return Ok(SocketAddr::V6(SocketAddrV6::new(ip, port, 0, 0)));
        }
        _ => {}
    }

    return Err(io::Error::new(io::ErrorKind::Other, "can't parse address"));
}

// func ToAddr(addr string) []byte {
// 	if strings.Index(addr, ":") < 0 {
// 		addr += ":80"
// 	}

// 	host, port, err := net.SplitHostPort(addr) //stats.g.doubleclick.net:443
// 	if err != nil {
// 		return nil
// 	}
// 	addrBytes := make([]byte, 0)
// 	ip := net.ParseIP(host)

// 	if ip == nil {
// 		l := len(host)
// 		addrBytes = append(addrBytes, DOMAIN_ADDR)
// 		addrBytes = append(addrBytes, byte(l))
// 		addrBytes = append(addrBytes, []byte(host)...)
// 	} else if len(ip) == 4 {
// 		addrBytes = append(addrBytes, IPV4_ADDR)
// 		addrBytes = append(addrBytes, []byte(ip)...)
// 	} else if len(ip) == 16 {
// 		addrBytes = append(addrBytes, IPV6_ADDR)
// 		addrBytes = append(addrBytes, []byte(ip)...)
// 	}
// 	p, err := strconv.Atoi(port)
// 	if err != nil {
// 		return nil
// 	}

// 	bp := make([]byte, 2)
// 	binary.BigEndian.PutUint16(bp, uint16(p))

// 	addrBytes = append(addrBytes, bp...)
// 	return addrBytes
// }

use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::str;

const debug: bool = true;

const IPv4Addr: u8 = 0x1;
const DomainAddr: u8 = 0x3;
const IPv6Addr: u8 = 0x4;

const ipv4Len: usize = 4;
const ipv6Len: usize = 16;
const portLen: usize = 2;

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
    let mut raw = [0u8; 260];

    let mut pos = 1;
    r.read_exact(&mut raw[0..pos]);

    let atyp = raw[0];
    if debug {
        println!("ATYP {}", atyp);
    }

    let mut rawAddrLen = portLen;
    match atyp {
        IPv4Addr => {
            rawAddrLen = rawAddrLen + ipv4Len;
        }
        DomainAddr => {
            r.read_exact(&mut raw[pos..pos + 1]);
            rawAddrLen = rawAddrLen + raw[pos] as usize;
            pos = pos + 1;
        }
        IPv6Addr => {
            rawAddrLen = rawAddrLen + ipv6Len;
        }
        _ => {
            println!("unsupported address type");
            return Err(io::Error::new(io::ErrorKind::Other, "unsupported address type"));
        }
    }

    let mut rawAddr = [0u8; 260];
    r.read_exact(&mut rawAddr[0..rawAddrLen]);

    let port = rawAddr[rawAddrLen - portLen] as u16 * 256 + rawAddr[rawAddrLen - 1] as u16;
    println!("Port {}", port);
    match atyp {
        IPv4Addr => {
            let ip = Ipv4Addr::new(rawAddr[0], rawAddr[1], rawAddr[2], rawAddr[3]);
            println!("{}", ip);
            return Ok(SocketAddr::V4(SocketAddrV4::new(ip, port)));
        }
        DomainAddr => {
            let host = str::from_utf8(&rawAddr[0..rawAddrLen - portLen]).unwrap().to_string();
            println!("DomainAddr {}", host);
        }
        IPv6Addr => {
            let ip = Ipv6Addr::new(rawAddr[0] as u16 * 256 + rawAddr[1] as u16,
                                   rawAddr[2] as u16 * 256 + rawAddr[3] as u16,
                                   rawAddr[4] as u16 * 256 + rawAddr[5] as u16,
                                   rawAddr[6] as u16 * 256 + rawAddr[7] as u16,
                                   rawAddr[8] as u16 * 256 + rawAddr[9] as u16,
                                   rawAddr[10] as u16 * 256 + rawAddr[11] as u16,
                                   rawAddr[12] as u16 * 256 + rawAddr[13] as u16,
                                   rawAddr[14] as u16 * 256 + rawAddr[15] as u16);
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
// 		addrBytes = append(addrBytes, DomainAddr)
// 		addrBytes = append(addrBytes, byte(l))
// 		addrBytes = append(addrBytes, []byte(host)...)
// 	} else if len(ip) == 4 {
// 		addrBytes = append(addrBytes, IPv4Addr)
// 		addrBytes = append(addrBytes, []byte(ip)...)
// 	} else if len(ip) == 16 {
// 		addrBytes = append(addrBytes, IPv6Addr)
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

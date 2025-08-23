use std::net::Ipv4Addr;

pub const TCP_START_PORT: u16 = 8000;
pub const TCP_END_PORT: u16 = 9000;
pub const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 0);
pub const MULTICAST_PORT: u16 = 7784;
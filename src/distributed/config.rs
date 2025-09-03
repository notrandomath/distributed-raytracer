use std::net::{Ipv4Addr, SocketAddrV4};

pub const TCP_START_PORT: u16 = 8000;
pub const TCP_END_PORT: u16 = 9000;

pub const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 0);
pub const MULTICAST_PORT: u16 = 7784;

pub const ORCHESTRATOR_CLIENT_CONNECTION_SOCKET: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 27301);
pub const ORCHESTRATOR_SERVER_CONNECTION_SOCKET: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 27302);

pub const NUM_OBJ_SERVERS: i32 = 50;
pub const NUM_REPEAT_OBJECT: i32 = 10;
pub const NUM_RAY_SERVERS: i32 = 5;
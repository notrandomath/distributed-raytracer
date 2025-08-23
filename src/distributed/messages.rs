use bincode::{Encode, Decode};
use std::net::Ipv4Addr;
use std::fmt::{Display, Formatter, Result};

#[derive(Encode, Decode, Debug)]
pub enum ServerType {
    Ray,
    Object
}

impl Display for ServerType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ServerType::Ray => write!(f, "Ray"),
            ServerType::Object => write!(f, "Object"),
        }
    }
}

#[derive(Encode, Decode, Debug)]
pub struct ServerDiscoveryMessage {
    pub server_type: ServerType,
    pub ip_address: Ipv4Addr,
    pub port: u16
}

impl Display for ServerDiscoveryMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Write the vector components separated by spaces.
        write!(f, "{} {} {}", self.server_type, self.ip_address, self.port)
    }
}

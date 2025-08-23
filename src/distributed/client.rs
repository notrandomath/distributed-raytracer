use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT};
use crate::distributed::messages::ServerDiscoveryMessage;
use std::io::Result;

pub fn run_client() -> Result<()> {
    // Bind to the socket that will receive the multicast packets
    let socket = UdpSocket::bind(SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT))?;
    
    // Join the multicast group on the local interface (0.0.0.0)
    socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
    
    println!("Joined multicast group and listening for messages...");
    
    let mut buf = [0; 256];
    loop {
        // Receive data from the socket
        let (num_bytes, src_addr) = socket.recv_from(&mut buf)?;
        
        let (port, _num_bytes_decoded): (ServerDiscoveryMessage, usize) = bincode::decode_from_slice(
            &buf[..num_bytes], bincode::config::standard()).unwrap();
        println!("Received from {}: {}", src_addr, port);
    }
}
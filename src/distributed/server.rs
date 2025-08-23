use std::net::{TcpListener, UdpSocket, Ipv4Addr, SocketAddrV4};
use std::io::{Result, Write};
use std::thread;
use bincode;
use std::time::Duration;
use crate::distributed::config::{TCP_START_PORT, TCP_END_PORT, MULTICAST_ADDR, MULTICAST_PORT};
use crate::distributed::messages::{ServerType, ServerDiscoveryMessage};

fn find_available_port_in_range(start_port: u16, end_port: u16) -> Option<u16> {
    for port in start_port..=end_port {
        // Attempt to bind to the port.
        if let Ok(_listener) = TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
            // If the bind is successful, the port is available.
            // We can immediately drop the listener to free the port for our main server.
            println!("✅ Found an available port: {}", port);
            return Some(port);
        }
    }
    // If the loop completes without finding an available port, return None.
    None
}

// This function handles incoming client connections
fn run_tcp_server(port: u16) -> Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    println!("TCP server listening on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New client connected!");
                // Echo a message back to the client
                let _ = stream.write_all(b"Hello, client!");
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
    Ok(())
}

// This function multicasts the server's port number
fn multicast_port_announcer(port_to_announce: u16) -> Result<()> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    socket.set_multicast_loop_v4(true)?; // Since server and client are both on localhost
    
    let multicast_addr = SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT);

    println!("Multicasting port {} to {}", port_to_announce, multicast_addr);

    let message = ServerDiscoveryMessage{
        server_type: ServerType::Ray,
        ip_address: Ipv4Addr::LOCALHOST,
        port: port_to_announce
    };
    
    let message_bytes: Vec<u8> = bincode::encode_to_vec(&message, 
        bincode::config::standard()).unwrap();

    loop {
        socket.send_to(&message_bytes, multicast_addr)?;
        thread::sleep(Duration::from_secs(3)); // Announce every 3 seconds
    }
}

pub fn run_server() {
    let tcp_port = find_available_port_in_range(TCP_START_PORT, TCP_END_PORT)
        .expect("No available port found");

    // Start the TCP server in a separate thread.
    let tcp_handle = thread::spawn(move || {
        if let Err(e) = run_tcp_server(tcp_port) {
            eprintln!("TCP server error: {}", e);
        }
    });

    // Start the multicast announcer in a separate thread.
    let multicast_handle = thread::spawn(move || {
        if let Err(e) = multicast_port_announcer(tcp_port) {
            eprintln!("Multicast announcer error: {}", e);
        }
    });

    // Wait for both threads to finish (which they won't, as they run infinitely).
    let _ = tcp_handle.join();
    let _ = multicast_handle.join();
}
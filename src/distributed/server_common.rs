use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket, TcpStream};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::Duration;
use std::io::{Result, Read, Write};
use serde::Serialize;
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT};
use crate::distributed::messages::{ServerDiscoveryMessage, ServerType};
use crate::distributed::{ray_server::RayServer, object_server::ObjectServer, orchestrator_server::OrchestratorServer};

pub fn send_tcp_message<T: Serialize>(socket_addr: &SocketAddrV4, message: &T) -> Result<String> {    
    // 1. Establish the connection
    let mut stream = match TcpStream::connect(socket_addr) {
        Ok(s) => {
            s
        },
        Err(e) => {
            eprintln!("🛑 Failed to connect: {}", e);
            return Err(e); // Exit the function with the connection error
        }
    };

    // 2. Write data to the server
    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&message, 
        bincode::config::standard()).unwrap();
    
    // Write all bytes to the stream
    stream.write_all(message_bytes.as_slice())?;
    
    // 3. Read the server's response (e.g., an echo)
    let mut buffer = [0; 128]; // Buffer to hold the response
    
    // Read the response into the buffer
    let bytes_read = stream.read(&mut buffer)?;
    
    // Convert the received bytes into a readable string
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);

    Ok(response.into_owned())
}

// This function multicasts the server's port number
fn multicast_port_announcer(
    port_to_announce: u16,
    server_type: ServerType, 
    should_stop: Arc<AtomicBool>
) -> Result<()> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    socket.set_multicast_loop_v4(true)?; // Since server and client are both on localhost
    
    let multicast_addr = SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT);

    println!("Multicasting port {} to {}", port_to_announce, multicast_addr);

    let message = ServerDiscoveryMessage{
        server_type: server_type,
        socket_addr: SocketAddrV4::new(Ipv4Addr::LOCALHOST, port_to_announce)
    };
    
    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&message, 
        bincode::config::standard()).unwrap();

    loop {
        if should_stop.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(3)); // Check every 3 seconds
        } else {
            socket.send_to(&message_bytes, multicast_addr)?;
            thread::sleep(Duration::from_secs(3)); // Announce every 3 seconds
        }
    }
}

pub fn run_server(port: u16, is_object_server: bool) {
    let should_stop = Arc::new(AtomicBool::new(false));
    let multicast_stop_flag = Arc::clone(&should_stop);

    
    // Start the TCP server in a separate thread.
    let tcp_handle = if is_object_server {
        thread::spawn(move || {
            if let Err(e) = ObjectServer::new(Arc::clone(&should_stop)).run_tcp_server(port) {
                eprintln!("TCP server error: {}", e);
            }
        })
    } else {
        thread::spawn(move || {
            if let Err(e) = ObjectServer::new(Arc::clone(&should_stop)).run_tcp_server(port) {
                eprintln!("TCP server error: {}", e);
            }
        })
    };

    // Start the multicast announcer in a separate thread.
    let multicast_handle = thread::spawn(move || {
        if let Err(e) = multicast_port_announcer(port, ServerType::Object,multicast_stop_flag) {
            eprintln!("Multicast announcer error: {}", e);
        }
    });

    // Wait for both threads to finish (which they won't, as they run infinitely).
    let _ = tcp_handle.join();
    let _ = multicast_handle.join();
}
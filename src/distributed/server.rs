use std::net::{TcpListener, UdpSocket, Ipv4Addr, SocketAddrV4};
use std::io::{Result, Write};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use bincode;
use std::time::Duration;
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT};
use crate::distributed::messages::{ServerType, ServerDiscoveryMessage};

// This function handles incoming client connections
fn run_tcp_server(port: u16, should_stop: Arc<AtomicBool>) -> Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
    println!("TCP server listening on port {}", port);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("New client connected! Stopping Multicast");
                should_stop.store(true, Ordering::SeqCst);

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
fn multicast_port_announcer(port_to_announce: u16, should_stop: Arc<AtomicBool>) -> Result<()> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))?;
    socket.set_multicast_loop_v4(true)?; // Since server and client are both on localhost
    
    let multicast_addr = SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT);

    println!("Multicasting port {} to {}", port_to_announce, multicast_addr);

    let message = ServerDiscoveryMessage{
        server_type: ServerType::Ray,
        socket_addr: SocketAddrV4::new(Ipv4Addr::LOCALHOST, port_to_announce)
    };
    
    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&message, 
        bincode::config::standard()).unwrap();

    loop {
        if should_stop.load(Ordering::SeqCst) {
            println!("Multicast announcer received stop signal. Exiting thread.");
            break; 
        }

        socket.send_to(&message_bytes, multicast_addr)?;
        thread::sleep(Duration::from_secs(3)); // Announce every 3 seconds
    }
    Ok(())
}

pub fn run_server(tcp_port: u16) {
    let should_stop = Arc::new(AtomicBool::new(false));
    let tcp_stop_flag = Arc::clone(&should_stop);
    let multicast_stop_flag = Arc::clone(&should_stop);

    // Start the TCP server in a separate thread.
    let tcp_handle = thread::spawn(move || {
        if let Err(e) = run_tcp_server(tcp_port, tcp_stop_flag) {
            eprintln!("TCP server error: {}", e);
        }
    });

    // Start the multicast announcer in a separate thread.
    let multicast_handle = thread::spawn(move || {
        if let Err(e) = multicast_port_announcer(tcp_port, multicast_stop_flag) {
            eprintln!("Multicast announcer error: {}", e);
        }
    });

    // Wait for both threads to finish (which they won't, as they run infinitely).
    let _ = tcp_handle.join();
    let _ = multicast_handle.join();
}
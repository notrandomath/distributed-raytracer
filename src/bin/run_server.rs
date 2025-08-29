use dray_lib::distributed::server::run_server;
use dray_lib::distributed::config::{TCP_START_PORT, TCP_END_PORT};
use std::thread::sleep;
use std::time::Duration;
use std::{net::{Ipv4Addr, TcpListener}, thread};


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

fn main() {
    let mut handles = vec![];

    for i in 0..3 {
        let handle = thread::spawn(move || {
            println!("Starting server thread #{}", i);
            run_server(find_available_port_in_range(TCP_START_PORT, TCP_END_PORT).expect("No available port found"));
        });
        sleep(Duration::from_secs_f32(0.5));
        handles.push(handle);
    }

    // Wait for all server threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}
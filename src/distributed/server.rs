use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, UdpSocket};
use std::io::{Read, Result, Write};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use bincode;
use std::time::Duration;
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT};
use crate::distributed::messages::{
    ObjectServerMessage, 
    ObjectServerMessageType, 
    ServerDiscoveryMessage, 
    ServerType
};
use crate::raytracer::hittable_list::HittableList;

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

pub struct ObjectServer{
    objects: HittableList,
    should_stop: Arc<AtomicBool>,
}

impl ObjectServer {
    pub fn new(should_stop: Arc<AtomicBool>) -> Self {
        ObjectServer {
            objects: HittableList::new(),
            should_stop
        }
    }

    fn handle_msg(&mut self, msg: &mut ObjectServerMessage) {
        match msg.message_type {
            ObjectServerMessageType::Deregistration => {
                self.should_stop.store(true, Ordering::SeqCst);
            }
            ObjectServerMessageType::Registration => {
                self.should_stop.store(false, Ordering::SeqCst);
            }
            ObjectServerMessageType::AddObject => {
                self.objects.add(msg.object_add.clone().unwrap());
            }
            ObjectServerMessageType::CheckHit => {
            }
            ObjectServerMessageType::PrintObjects => {
                println!("Num Objects: {}", self.objects.len())
            }
        }
    }

    fn run_tcp_server(&mut self, port: u16) -> Result<()> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        println!("TCP server listening on port {}", port);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    // Read the response into the buffer
                    let mut buf = [0; 128];
                    let num_bytes = stream.read(&mut buf)?;
                    // Convert the bytes into a decoded server message
                    let (mut msg, _num_bytes_decoded): (ObjectServerMessage, usize) = bincode::serde::decode_from_slice(
                        &buf[..num_bytes], bincode::config::standard()).unwrap();
                    self.handle_msg(&mut msg);
                    // Writes new message (msg was modified by self.handle_msg)
                    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&msg, 
                        bincode::config::standard()).unwrap();
                    stream.write_all(message_bytes.as_slice())?;
                    
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
        Ok(())
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
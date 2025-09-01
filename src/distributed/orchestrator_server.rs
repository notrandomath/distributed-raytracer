use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::io::{ErrorKind, Read, Result, Write};
use std::rc::Rc;
use bincode;
use crate::distributed::messages::*;
use crate::distributed::server_common::send_tcp_message;
use crate::raytracer::bounding_box::BoundingBox;
use crate::raytracer::hittable_list::HittableList;
use std::collections::HashMap;
use std::time::Duration;
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT};
use std::sync::Arc;
use futures_util::{FutureExt, SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;

pub async fn run_orchestrator(addr: SocketAddrV4) {
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Accept new connections in a loop.
    while let Ok((stream, peer_addr)) = listener.accept().await {
        // Spawn a new asynchronous task for each connection.
        // The `spawn` function returns a `JoinHandle` which we don't need to await here.
        let mut orchestrator = OrchestratorServer::new();
        orchestrator.discover_servers().unwrap();
        orchestrator.create_bounding_volumes();
        tokio::spawn(async move {
            orchestrator.handle_connection(stream, peer_addr).await;
        });
    }
}

pub struct OrchestratorServer{
    server_directory: [Vec<SocketAddrV4>; NUM_SERVER_TYPES],
    boxes: Vec<Arc<BoundingBox>>,
    boxes_hittable: HittableList,
    object_map: HashMap<usize, SocketAddrV4>,
}

impl OrchestratorServer {
    pub fn new() -> Self {
        OrchestratorServer {
            server_directory: std::array::from_fn(|_| Vec::new()),
            boxes: Vec::new(),
            boxes_hittable: HittableList::new(),
            object_map: HashMap::new(),
        }
    }

    async fn handle_connection(&mut self, stream: TcpStream, peer_addr: SocketAddr) {
        println!("New WebSocket connection from: {}", peer_addr);

        // The `accept_async` method performs the WebSocket handshake.
        let ws_stream = tokio_tungstenite::accept_async(stream)
            .await
            .expect("Error during the websocket handshake");

        // Split the stream into a sender and a receiver.
        let (mut write, mut read) = ws_stream.split();

        // Loop to read incoming messages from the client.
        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(_)) => {}
                Ok(Message::Binary(binary)) => {
                    let (mut msg, _num_bytes_decoded): (OrchestratorServerMessage, usize) = bincode::serde::decode_from_slice(
                        &binary, bincode::config::standard()).unwrap();
                    self.handle_msg(&mut msg);
                }
                Ok(Message::Ping(_)) => {}
                Ok(Message::Close(close_frame)) => {
                    println!("Received a close message from {}: {:?}", peer_addr, close_frame);
                    // The stream will be closed automatically when the handler exits.
                    break;
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Frame(_)) => {}
                Err(e) => {
                    eprintln!("Error receiving message from {}: {}", peer_addr, e);
                    break;
                }
            }
        }

        println!("WebSocket connection closed for: {}", peer_addr);
    }

    fn create_bounding_volumes(&mut self) {
        let n = self.server_directory[ServerType::Object as usize].len();
        let mut cur_i: usize = 0;
        for a in (-10..=10).step_by(4) {
            for b in (-10..=10).step_by(4) {
                let bv = BoundingBox::new_xyz(
                    if a == -10 {-1e6} else {(a as f64) - 4.},
                    if a == 10 {1e6} else {(a as f64) + 4.},
                    -1e6,
                    1e6,
                    if b == -10 {-1e6} else {(b as f64) - 4.},
                    if b == 10 {1e6} else {(b as f64) + 4.},
                );
                self.object_map.insert(self.boxes.len(), self.server_directory[ServerType::Object as usize][cur_i]);
                let new_box = Arc::new(bv);
                self.boxes.push(new_box.clone());
                self.boxes_hittable.add(new_box.clone());
                cur_i = (cur_i+1)%n;
            }
        }
    }

    fn handle_msg(&mut self, msg: &mut OrchestratorServerMessage) {
        match msg.message_type {
            OrchestratorServerMessageType::SendObject => {
                for (index, aabb) in self.boxes.iter().enumerate() {
                    let new_sphere = msg.object.clone().unwrap();
                    if aabb.intersect_sphere(&new_sphere) {
                        let _ = send_tcp_message(
                            &self.object_map[&index], 
                            &ObjectServerMessage::new_object_add(new_sphere.clone())
                        );
                    }
                }
            }
            OrchestratorServerMessageType::BeginRaytracing => {
                let _ = self.run_raytracer();
            }
            OrchestratorServerMessageType::ReceivePixel => {
                panic!("Orchestrator Server should not receive pixels from itself") 
            }
        }
    }

    fn discover_servers(&mut self) -> Result<()> {
        // Bind to the socket that will receive the multicast packets
        let socket = UdpSocket::bind(SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT))?;
        
        // Join the multicast group on the local interface (0.0.0.0)
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        socket.set_read_timeout(Some(Duration::from_secs(10)))?;
        
        println!("Joined multicast group and listening for messages...");                 

        let mut buf = [0; 256];
        loop {
            // Receive data from the socket
            let recv_result = socket.recv_from(&mut buf);

            match recv_result {
                Ok((num_bytes, _src_addr)) => {
                    let (msg, _num_bytes_decoded): (ServerDiscoveryMessage, usize) = bincode::serde::decode_from_slice(
                        &buf[..num_bytes], bincode::config::standard()).unwrap();
                    if !self.server_directory[msg.server_type as usize].contains(&msg.socket_addr) {
                        self.server_directory[msg.server_type as usize].push(msg.socket_addr);
                        send_tcp_message(&msg.socket_addr, &ObjectServerMessage::new_no_data(ObjectServerMessageType::Deregistration))?;
                    }
                }
                // Error: Check if it's a timeout error
                Err(ref e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => {
                    println!("\nTimeout reached.");
                    break;
                },
                // Other error
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        println!("\nFinal Server Directory:");
        for (i, set) in self.server_directory.iter().enumerate() {
            println!("Type {}: {} servers found", i, set.len());
            for addr in set.iter() {
                println!("  - {}", addr);
            }
        }

        Ok(())
    }

    fn run_raytracer(&mut self) -> Result<()> {
        println!("Printing objects...");
        for addr in self.server_directory[ServerType::Object as usize].iter() {
            let _result = send_tcp_message(
                addr, 
                &ObjectServerMessage::new_no_data(ObjectServerMessageType::PrintObjects)
            );
        }

        Ok(())
    }
}
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::io::{ErrorKind, Result};
use bincode;
use futures_util::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use crate::distributed::messages::*;
use crate::distributed::distributed_common::{run_async_server, send_tcp_message, send_websocket_message};
use crate::raytracer::bounding_box::BoundingBox;
use crate::raytracer::camera::{Camera};
use std::collections::HashMap;
use std::time::Duration;
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT, NUM_REPEAT_OBJECT, ORCHESTRATOR_CLIENT_CONNECTION_SOCKET, ORCHESTRATOR_SERVER_CONNECTION_SOCKET};
use std::sync::Arc;
use tokio;
use futures_util::{StreamExt};
use tokio_tungstenite::tungstenite::Message;

pub async fn run_orchestrator() {
    let try_socket = tokio::net::TcpListener::bind(&ORCHESTRATOR_CLIENT_CONNECTION_SOCKET).await;
    let listener = try_socket.expect("Failed to bind");

    // Accept new connections in a loop.
    while let Ok((stream, peer_addr)) = listener.accept().await {
        let (tx, rx) = tokio::sync::mpsc::channel::<OrchestratorServerMessage>(128);
        tokio::spawn(
            run_async_server(
                ORCHESTRATOR_SERVER_CONNECTION_SOCKET,
                move |msg: &OrchestratorServerMessage| {
                    // Clone the Arc to create a new shared reference for this call
                    let tx_clone = tx.clone();
                    let cloned_msg = msg.clone(); 
                    async move {
                        let _ = tx_clone.send(cloned_msg.clone()).await.unwrap();
                        cloned_msg
                    }
                }
            )
        );

        // Spawn a new asynchronous task for each connection.
        // The `spawn` function returns a `JoinHandle` which we don't need to await here.
        let mut orchestrator = OrchestratorServer::new(rx);
        orchestrator.discover_servers().await.unwrap();
        orchestrator.create_bounding_volumes();
        tokio::spawn(async move {
            orchestrator.handle_connection(stream, peer_addr).await;
        });
    }
}

pub struct OrchestratorServer{
    rx: tokio::sync::mpsc::Receiver<OrchestratorServerMessage>,
    server_directory: [Vec<SocketAddrV4>; NUM_SERVER_TYPES],
    boxes: Vec<Arc<BoundingBox>>,
    box_map: HashMap<usize, Vec<SocketAddrV4>>,
    camera: Camera
}

async fn distribute_rays(camera: Camera, server_directory: [Vec<SocketAddrV4>; NUM_SERVER_TYPES]) {
    for (ray_index, ray) in camera.iterate_rays() {
        let consolidated_idx = ray_index.pixel_i+ray_index.pixel_j+ray_index.pixel_sample_num;
        let server_idx = (consolidated_idx as usize) % server_directory[ServerType::Ray as usize].len();
        let _ = send_tcp_message(
            &server_directory[ServerType::Ray as usize][server_idx], 
            &RayServerMessage::new_share_ray(&ray_index, &ray)
        ).await;
    }
}

impl OrchestratorServer {
    pub fn new(rx: tokio::sync::mpsc::Receiver<OrchestratorServerMessage>) -> Self {
        OrchestratorServer {
            rx: rx,
            server_directory: std::array::from_fn(|_| Vec::new()),
            boxes: Vec::new(),
            box_map: HashMap::new(),
            camera: Camera::default()
        }
    }

    async fn handle_connection(&mut self, stream: tokio::net::TcpStream, peer_addr: SocketAddr) {
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
                    self.handle_msg(&mut write, &mut msg).await;
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
        for _ in 0..NUM_REPEAT_OBJECT {
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
                    if !self.box_map.contains_key(&self.boxes.len()) {
                        self.box_map.insert(self.boxes.len(), Vec::new());
                    }
                    self.box_map.get_mut(&self.boxes.len()).unwrap().push(self.server_directory[ServerType::Object as usize][cur_i]);
                    let new_box = Arc::new(bv);
                    self.boxes.push(new_box.clone());
                    cur_i = (cur_i+1)%n;
                }
            }
        }
    }

    async fn handle_msg(
        &mut self, 
        write: &mut SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>,
        msg: &mut OrchestratorServerMessage
    ) {
        match msg.message_type {
            OrchestratorServerMessageType::SendObject => {
                for (index, aabb) in self.boxes.iter().enumerate() {
                    let new_sphere = msg.object.clone().unwrap();
                    if aabb.intersect_sphere(&new_sphere) {
                        for address in self.box_map[&index].iter() {
                            let _ = send_tcp_message(
                                address, 
                                &ObjectServerMessage::new_object_add(new_sphere.clone())
                            ).await;
                        }
                    }
                }
            }
            OrchestratorServerMessageType::BeginRaytracing => {
                self.camera = msg.camera.clone().unwrap();
                let _ = self.run_raytracer(write).await;
            }
            OrchestratorServerMessageType::ReceivePixel => {
                panic!("Orchestrator Server should not receive pixels from itself") 
            }
        }
    }

    async fn discover_servers(&mut self) -> Result<()> {
        // Bind to the socket that will receive the multicast packets
        let socket = UdpSocket::bind(SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT))?;
        
        // Join the multicast group on the local interface (0.0.0.0)
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        socket.set_read_timeout(Some(Duration::from_secs(5)))?;
        
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
                        if msg.server_type == ServerType::Ray {
                            send_tcp_message(&msg.socket_addr, &RayServerMessage::new_no_data(RayServerMessageType::Deregistration)).await?;
                        } else {
                            send_tcp_message(&msg.socket_addr, &ObjectServerMessage::new_no_data(ObjectServerMessageType::Deregistration)).await?;
                        };
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

    async fn share_params(&self) {
        for i in 0..self.server_directory[ServerType::Ray as usize].len() {
            let _ = send_tcp_message(
                &self.server_directory[ServerType::Ray as usize][i], 
                &RayServerMessage::new_share_params(&self.boxes, &self.box_map, &self.camera)
            ).await;
        }
    }

    async fn run_raytracer(&mut self, 
        write: &mut SplitSink<WebSocketStream<tokio::net::TcpStream>, Message>
    ) -> Result<()> {
        println!("Printing objects...");
        for addr in self.server_directory[ServerType::Object as usize].iter() {
            let _result = send_tcp_message(
                addr, 
                &ObjectServerMessage::new_no_data(ObjectServerMessageType::PrintObjects)
            ).await;
        }

        println!("Sharing parameters...");
        self.share_params().await;

        println!("Distributing rays...");
        let thread_camera = self.camera.clone();
        let thread_server_directory = self.server_directory.clone();
        let _ = tokio::spawn(distribute_rays(thread_camera, thread_server_directory));
        
        println!("Waiting for ray responses...");
        while let Some(msg) = self.rx.recv().await {
            let _ = send_websocket_message(write, &msg).await;
        }

        Ok(())
    }
}
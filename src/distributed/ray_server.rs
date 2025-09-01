use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::io::{Read, Result, Write};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::thread;
use bincode;
use crate::distributed::messages::{
    RayServerMessage, RayServerMessageType
};
use crate::raytracer::camera::{PixelIndexEntry, RayColorEntry};
use crate::raytracer::bounding_box::BoundingBox;
use crate::raytracer::prelude::*;
use std::collections::HashMap;

pub struct RayServer{
    ray_entries: HashMap<PixelIndexEntry, RayColorEntry>,
    bounding_boxes: Vec<Arc<BoundingBox>>,
    object_servers: HashMap<usize, SocketAddrV4>,
    should_stop: Arc<AtomicBool>,
}

impl RayServer {
    pub fn new(should_stop: Arc<AtomicBool>) -> Self {
        RayServer {
            ray_entries: HashMap::new(),
            bounding_boxes: Vec::new(),
            object_servers: HashMap::new(),
            should_stop
        }
    }

    fn handle_msg(&mut self, msg: &mut RayServerMessage) {
        match msg.message_type {
            RayServerMessageType::Deregistration => {
                self.should_stop.store(true, Ordering::SeqCst);
            }
            RayServerMessageType::Registration => {
                self.should_stop.store(false, Ordering::SeqCst);
            }
            RayServerMessageType::SendObjectServerDirectory => {
                self.object_servers = msg.object_servers.clone().unwrap();
                self.bounding_boxes = msg.object_bbs.clone().unwrap();
            }
            RayServerMessageType::SendPixel => {
            }
            RayServerMessageType::CheckHit => {
            }
        }
    }

    pub fn run_tcp_server(&mut self, port: u16) -> Result<()> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, port))?;
        println!("TCP server listening on port {}", port);

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    // Read the response into the buffer
                    let mut buf = [0; 128];
                    let num_bytes = stream.read(&mut buf)?;
                    // Convert the bytes into a decoded server message
                    let (mut msg, _num_bytes_decoded): (RayServerMessage, usize) = bincode::serde::decode_from_slice(
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
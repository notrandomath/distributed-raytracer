use std::net::{Ipv4Addr, TcpListener};
use std::io::{Read, Result, Write};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use bincode;
use crate::distributed::messages::{
    ObjectServerMessage, 
    ObjectServerMessageType, 
};
use crate::raytracer::hittable_list::HittableList;

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
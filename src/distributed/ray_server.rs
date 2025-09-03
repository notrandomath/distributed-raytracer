use core::time;
use std::net::{SocketAddrV4};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::mpsc;
use bincode;
use tokio::time::sleep;
use crate::distributed::config::ORCHESTRATOR_SERVER_CONNECTION_SOCKET;
use crate::distributed::messages::{
    ObjectServerMessage, OrchestratorServerMessage, RayServerMessage, RayServerMessageType
};
use crate::distributed::distributed_common::send_tcp_message;
use crate::raytracer::camera::{Camera, PixelIndexEntry, RayColorEntry, RayColorStatus};
use crate::raytracer::bounding_box::BoundingBox;
use crate::raytracer::hittable::{HitRecord, Hittable};
use crate::raytracer::hittable_list::HittableList;
use crate::raytracer::prelude::*;
use std::collections::HashMap;

struct RayProcessor {
    ray_entries: HashMap<PixelIndexEntry, RayColorEntry>,
    bounding_boxes: HittableList,
    object_servers: HashMap<usize, Vec<SocketAddrV4>>,
    camera: Camera,
    rx: mpsc::Receiver<(PixelIndexEntry, Ray)>
}

impl RayProcessor {
    pub fn new(
        bounding_boxes: Vec<Arc<BoundingBox>>,
        object_servers: HashMap<usize, Vec<SocketAddrV4>>,
        camera: Camera,
        rx: mpsc::Receiver<(PixelIndexEntry, Ray)>
    ) -> Self {
        RayProcessor {
            ray_entries: HashMap::new(),
            bounding_boxes: HittableList::new_w_objs(bounding_boxes
                .into_iter()
                .map(|obj_arc| -> Arc<dyn Hittable> { obj_arc })
                .collect()),
            object_servers: object_servers,
            camera: camera,
            rx: rx
        }
    }

    pub async fn run(&mut self) {
        while let Some((pixel_idx, ray)) = self.rx.recv().await {
            if !self.ray_entries.contains_key(&pixel_idx) {
                self.ray_entries.insert(pixel_idx.clone(), 
                RayColorEntry::new(ray.clone(), self.camera.max_depth));
            }
            loop {
                let mut finished: bool = true;
                let mut status: RayColorStatus = RayColorStatus::default();
                let mut first_hit: RayColorEntry = self.ray_entries[&pixel_idx].clone();
                for (aabb_idx, _distance) in self.bounding_boxes.hits_vec(
                    &self.ray_entries[&pixel_idx].ray, 
                    Interval::new_min_max(0.001, INFINITY), 
                    &mut HitRecord::default()).iter() 
                {
                    let mut server_idx: usize = 0;
                    loop {
                        let response = send_tcp_message(
                            &self.object_servers[&aabb_idx][server_idx], 
                            &ObjectServerMessage::new_ray_check(self.ray_entries[&pixel_idx].clone()
                        )).await;
                        match response {
                            Ok(response_bytes) => {
                                let (msg, _num_bytes_decoded): (ObjectServerMessage, usize) = bincode::serde::decode_from_slice(
                                    &response_bytes, bincode::config::standard()).unwrap();
                                first_hit = msg.ray_entry.unwrap();
                                status = msg.ray_status.unwrap();
                                break;
                            }
                            Err(_) => {
                                if server_idx == self.object_servers[&aabb_idx].len()-1 {
                                    sleep(time::Duration::from_secs(5)).await;
                                    server_idx = 0;
                                } else {
                                    // timeout or some other error, so skip to other server that hosts object
                                    server_idx += 1;
                                }
                                
                            }
                        }
                    }

                    finished = status.finished & finished;
                    if status.hit_object_or_stop {
                        break;
                    }
                }
                self.ray_entries.insert(pixel_idx.clone(), first_hit);
                if finished {
                    let _ = send_tcp_message(
                        &ORCHESTRATOR_SERVER_CONNECTION_SOCKET, 
                        &OrchestratorServerMessage::new_pixel_response(
                            pixel_idx.clone(), 
                            self.ray_entries[&pixel_idx].color
                        )
                    ).await;
                    break; 
                }
            }
        }
    }
}

pub struct RayServer{
    tx: mpsc::Sender<(PixelIndexEntry, Ray)>,
    should_stop: Arc<AtomicBool>,
}

impl RayServer {
    pub fn new(should_stop: Arc<AtomicBool>) -> Self {
        RayServer {
            tx: mpsc::channel::<(PixelIndexEntry, Ray)>(128).0,
            should_stop: should_stop
        }
    }

    pub async fn handle_msg(&mut self, msg: &RayServerMessage) -> RayServerMessage {
        match msg.message_type {
            RayServerMessageType::Deregistration => {
                self.should_stop.store(true, Ordering::SeqCst);
            }
            RayServerMessageType::Registration => {
                self.should_stop.store(false, Ordering::SeqCst);
            }
            RayServerMessageType::SendObjectServerDirectory => {
                let (tx, rx) = mpsc::channel::<(PixelIndexEntry, Ray)>(128);
                
                let thread_msg = msg.clone();
                let _ = tokio::spawn(async move {
                    let mut ray_processor = RayProcessor::new(
                        thread_msg.object_bbs.clone().unwrap(),
                        thread_msg.object_servers.clone().unwrap(),
                        thread_msg.camera.clone().unwrap(),
                        rx
                    );
                    ray_processor.run().await;
                });
                self.tx = tx;
            }
            RayServerMessageType::SendPixel => {
                let _ = self.tx.send((msg.clone().pixel_index.unwrap(), msg.clone().ray.unwrap())).await;
            }
            RayServerMessageType::CheckHit => {}
        }
        msg.clone()
    }
}
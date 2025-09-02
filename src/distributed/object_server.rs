use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::distributed::messages::{
    ObjectServerMessage, 
    ObjectServerMessageType, 
};
use crate::raytracer::camera::ray_color_iteration;
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

    pub async fn handle_msg(&mut self, msg: &ObjectServerMessage) -> ObjectServerMessage {
        let mut new_msg = msg.clone();
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
                let mut entry = msg.ray_entry.clone().unwrap();
                new_msg.ray_status = Some(ray_color_iteration(&mut entry, &self.objects));
                new_msg.ray_entry = Some(entry);
            }
            ObjectServerMessageType::PrintObjects => {
                println!("Num Objects: {}", self.objects.len())
            }
        }
        new_msg
    }
}
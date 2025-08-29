use serde::{Serialize, Deserialize};
use std::net::{SocketAddrV4};
use std::fmt::{Display, Formatter, Result};
use crate::raytracer::hittable::{HitRecord, Hittable};
use crate::raytracer::prelude::*;

// since variant_count is only on nightly
pub const NUM_SERVER_TYPES: usize = 2;
#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ServerType {
    Ray,
    Object
}

impl Display for ServerType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ServerType::Ray => write!(f, "Ray"),
            ServerType::Object => write!(f, "Object"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct ServerDiscoveryMessage {
    pub server_type: ServerType,
    pub socket_addr: SocketAddrV4
}

impl Display for ServerDiscoveryMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // Write the vector components separated by spaces.
        write!(f, "{} {}", self.server_type, self.socket_addr)
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ObjectServerMessageType {
    Deregistration,
    Registration,
    AddObject,
    CheckHit,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectServerMessage {
    pub message_type: ObjectServerMessageType,
    pub object_add: Option<Rc<dyn Hittable>>,
    pub ray: Option<Ray>,
    pub ray_t: Option<Interval>,
    pub hit_record: Option<HitRecord>,
    pub is_absorbed: Option<bool>
}

impl ObjectServerMessage {
    pub fn new_deregistration() -> Self {
        ObjectServerMessage {
            message_type: ObjectServerMessageType::Deregistration,
            object_add: None,
            ray: None,
            ray_t: None,
            hit_record: None,
            is_absorbed: None
        }
    }

    pub fn new_object_add(object: Rc<dyn Hittable>) -> Self {
        ObjectServerMessage {
            message_type: ObjectServerMessageType::AddObject,
            object_add: Some(object),
            ray: None,
            ray_t: None,
            hit_record: None,
            is_absorbed: None
        }
    }
}
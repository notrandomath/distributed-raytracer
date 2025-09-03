use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::net::{SocketAddrV4};
use std::fmt::{Display, Formatter, Result};
use crate::raytracer::bounding_box::BoundingBox;
use crate::raytracer::camera::{Camera, PixelIndexEntry, RayColorEntry, RayColorStatus};
use crate::raytracer::hittable::{Hittable};
use crate::raytracer::{prelude::*};
use crate::raytracer::sphere::Sphere;

// since variant_count is only on nightly
pub const NUM_SERVER_TYPES: usize = 2;
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
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

#[derive(Serialize, Deserialize, Clone)]
pub enum ObjectServerMessageType {
    Deregistration,
    Registration,
    AddObject,
    PrintObjects,
    CheckHit,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectServerMessage {
    pub message_type: ObjectServerMessageType,
    pub object_add: Option<Arc<dyn Hittable>>,
    pub ray_entry: Option<RayColorEntry>,
    pub ray_status: Option<RayColorStatus>,
}

impl ObjectServerMessage {
    pub fn new_no_data(message_type: ObjectServerMessageType) -> Self {
        ObjectServerMessage {
            message_type: message_type,
            object_add: None,
            ray_entry: None,
            ray_status: None
        }
    }

    pub fn new_object_add(object: Arc<dyn Hittable>) -> Self {
        ObjectServerMessage {
            message_type: ObjectServerMessageType::AddObject,
            object_add: Some(object),
            ray_entry: None,
            ray_status: None
        }
    }

    pub fn new_ray_check(ray_entry: RayColorEntry) -> Self {
        ObjectServerMessage {
            message_type: ObjectServerMessageType::CheckHit,
            object_add: None,
            ray_entry: Some(ray_entry),
            ray_status: None
        }
    }

    pub fn new_ray_check_response(ray_entry: RayColorEntry, ray_status: RayColorStatus) -> Self {
        ObjectServerMessage {
            message_type: ObjectServerMessageType::CheckHit,
            object_add: None,
            ray_entry: Some(ray_entry),
            ray_status: Some(ray_status)
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum RayServerMessageType {
    Deregistration,
    Registration,
    SendObjectServerDirectory,
    SendPixel,
    CheckHit,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RayServerMessage {
    pub message_type: RayServerMessageType,
    pub object_bbs: Option<Vec<Arc<BoundingBox>>>,
    pub object_servers: Option<HashMap<usize, Vec<SocketAddrV4>>>,
    pub camera: Option<Camera>,
    pub pixel_index: Option<PixelIndexEntry>,
    pub ray: Option<Ray>,
}

impl RayServerMessage {
    pub fn new_no_data(message_type: RayServerMessageType) -> Self {
        RayServerMessage {
            message_type: message_type,
            object_bbs: None,
            object_servers: None,
            camera: None,
            pixel_index: None,
            ray: None,
        }
    }

    pub fn new_share_params(
        object_bbs: &Vec<Arc<BoundingBox>>,
        server_directory: &HashMap<usize, Vec<SocketAddrV4>>,
        camera: &Camera
    ) -> Self {
        RayServerMessage {
            message_type: RayServerMessageType::SendObjectServerDirectory,
            object_bbs: Some(object_bbs.clone()),
            object_servers: Some(server_directory.clone()),
            camera: Some(camera.clone()),
            ray: None,
            pixel_index: None,
        }
    }

    pub fn new_share_ray(
        pixel_index: &PixelIndexEntry,
        ray: &Ray,
    ) -> Self {
        RayServerMessage {
            message_type: RayServerMessageType::SendPixel,
            object_bbs: None,
            object_servers: None,
            camera: None,
            pixel_index: Some(pixel_index.clone()),
            ray: Some(ray.clone()),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OrchestratorServerMessageType {
    SendObject,
    BeginRaytracing,
    ReceivePixel,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct OrchestratorServerMessage {
    pub message_type: OrchestratorServerMessageType,
    pub object: Option<Arc<Sphere>>,
    pub camera: Option<Camera>,
    pub pixel_index: Option<PixelIndexEntry>,
    pub pixel_color: Option<Color>
}

impl OrchestratorServerMessage {
    pub fn new_raytrace(camera: &Camera) -> Self {
        OrchestratorServerMessage {
            message_type: OrchestratorServerMessageType::BeginRaytracing,
            object: None,
            camera: Some(camera.clone()),
            pixel_index: None,
            pixel_color: None
        }
    }
    
    pub fn new_add_object(object: Arc<Sphere>) -> Self {
        OrchestratorServerMessage {
            message_type: OrchestratorServerMessageType::SendObject,
            object: Some(object),
            camera: None,
            pixel_index: None,
            pixel_color: None
        }
    }

    pub fn new_pixel_response(pixel_index: PixelIndexEntry, pixel_color: Color) -> Self {
        OrchestratorServerMessage {
            message_type: OrchestratorServerMessageType::SendObject,
            object: None,
            camera: None,
            pixel_index: Some(pixel_index),
            pixel_color: Some(pixel_color)
        }
    }
}
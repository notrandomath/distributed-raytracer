use crate::raytracer::vec3::{Vec3, Point3, dot};
use crate::raytracer::ray::Ray;
use crate::raytracer::prelude::*;
use crate::raytracer::material::{Material, DefaultMaterial};

#[derive(Clone)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Rc<dyn Material>,
    pub t: f64,
    pub front_face: bool
} 

impl HitRecord {
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        // Sets the hit record normal vector.
        // NOTE: the parameter `outward_normal` is assumed to have unit length.

        self.front_face = dot(r.direction(), outward_normal) < 0.;
        self.normal = if self.front_face { *outward_normal } else { (-1.) * *outward_normal };
    }
}

pub trait Hittable {
    fn hit(&self, r: &Ray, ray_t: Interval, hit_record: &mut HitRecord) -> bool;
}

impl Default for HitRecord {
    fn default() -> Self {
        // Provide a default implementation for HitRecord.
        // Note that `material` is set to `None` as there's no default `Material` trait object.
        HitRecord {
            p: Point3::default(),
            normal: Vec3::default(),
            mat: Rc::new(DefaultMaterial::default()),
            t: 0.0,
            front_face: false,
        }
    }
}
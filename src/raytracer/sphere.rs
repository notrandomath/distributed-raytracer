use crate::raytracer::prelude::*;
use crate::raytracer::hittable::{Hittable, HitRecord};
use crate::raytracer::material::Material;

#[derive(Serialize, Deserialize)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    mat: Arc<dyn Material>
}

impl Sphere {
    pub fn new(center: &Point3, radius: f64, mat: Arc<dyn Material>) -> Self {
        Sphere { center: *center, radius: f64::max(radius, 0.), mat}
    }

    pub fn center(&self) -> Vec3 {
        return self.center;
    }
    
    pub fn radius(&self) -> f64 {
        return self.radius;
    }
}

#[typetag::serde]
impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let oc: Vec3 = self.center - *r.origin();
        let a = r.direction().length_squared();
        let h = dot(r.direction(), &oc);
        let c = oc.length_squared() - self.radius*self.radius;

        let discriminant = h*h - a*c;
        
        if discriminant < 0. {
            return false;
        }

        let sqrtd = discriminant.sqrt();

        let mut root = (h - sqrtd) / a;
        if !ray_t.surrounds(root) {
            root = (h + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal: Vec3 = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, &outward_normal);
        rec.mat = self.mat.clone();

        return true;
    }
}
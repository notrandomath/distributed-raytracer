use crate::raytracer::material::Transparent;
use crate::raytracer::prelude::*;
use crate::raytracer::hittable::{Hittable, HitRecord};
use crate::raytracer::sphere::Sphere;

#[derive(Serialize, Deserialize)]
pub struct BoundingBox {
    axes: [Interval; 3]
}

impl BoundingBox {
    pub fn default() -> Self {
        BoundingBox { axes: [Interval::EMPTY; 3] }
    }

    pub fn new(axes: [Interval; 3]) -> Self {
        BoundingBox { axes }
    }

    pub fn new_xyz(x_min: f64, x_max: f64, y_min: f64, y_max: f64, z_min: f64, z_max: f64) -> Self {
        BoundingBox { 
            axes: [
                Interval::new_min_max(x_min, x_max),
                Interval::new_min_max(y_min, y_max),
                Interval::new_min_max(z_min, z_max)
            ]
        }
    }

    // Adapted from https://developer.mozilla.org/en-US/docs/Games/Techniques/3D_collision_detection 
    pub fn intersect_sphere(&self, sphere: &Sphere) -> bool {
        // get box closest point to sphere center by clamping
        let x = f64::max(self.axes[0].min, f64::min(sphere.center()[0], self.axes[0].max));
        let y = f64::max(self.axes[1].min, f64::min(sphere.center()[1], self.axes[1].max));
        let z = f64::max(self.axes[2].min, f64::min(sphere.center()[2], self.axes[2].max));

        // this is the same as isPointInsideSphere
        let distance: f64 = (
            (x - sphere.center()[0]) * (x - sphere.center()[0]) +
            (y - sphere.center()[1]) * (y - sphere.center()[1]) +
            (z - sphere.center()[2]) * (z - sphere.center()[2])
        ).sqrt();
        return distance < sphere.radius();
    }
}

#[typetag::serde]
impl Hittable for BoundingBox {
    // Adapted from https://tavianator.com/2011/ray_box.html
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let mut t_min: f64 = -INFINITY;
        let mut t_max: f64 = INFINITY;
        for a in 0..2 {
            if r.direction()[a] != 0.0 {
                let t0: f64 = self.axes[a].min - r.origin()[a] / r.direction()[a];
                let t1: f64 = self.axes[a].max - r.origin()[a] / r.direction()[a];
                t_min = f64::max(t_min, f64::min(t0, t1));
                t_max = f64::min(t_max, f64::max(t0, t1));
            }
        }
        if t_max >= t_min {
            if ray_t.surrounds(t_min) {
                rec.t = t_min;
                rec.p = r.at(t_min);
                rec.mat = Rc::new(Transparent{});
                return true;
            }
            if ray_t.surrounds(t_max) {
                rec.t = t_max;
                rec.p = r.at(t_max);
                rec.mat = Rc::new(Transparent{});
                return true;    
            }
        }
        false
    }
}
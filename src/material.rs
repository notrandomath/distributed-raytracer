use crate::prelude::*;
use crate::hittable::HitRecord;


pub trait Material {
    fn scatter(&self, _r_in: &Ray, _hit_record: &HitRecord, _attenuation: &mut Color, _scattered: &mut Ray) -> bool {
        // returns true if scattered otherwise false if absorbed
        return false;
    }
}

#[derive(Default)]
pub struct DefaultMaterial {}
impl Material for DefaultMaterial {}

pub struct Lambertian {
    albedo: Color
} 

impl Lambertian {
    pub fn new(albedo: &Color) -> Self {
        Lambertian { albedo: *albedo }
    }
}


impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        let mut scatter_direction = rec.normal + random_unit_vector();

        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new(rec.p, scatter_direction);
        *attenuation = self.albedo;
        return true;
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64
} 

impl Metal {
    pub fn new(albedo: &Color, fuzz: f64) -> Self {
        Metal { albedo: *albedo, fuzz: if fuzz < 1.0 { fuzz } else { 1.0 } }
    }
}


impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        let mut reflected: Vec3 = reflect(r_in.direction(), &rec.normal);
        reflected = unit_vector(&reflected) + (self.fuzz * random_unit_vector());
        *scattered = Ray::new(rec.p, reflected);
        *attenuation = self.albedo;
        // if the fuzzed reflection goes below the surface, absorb the ray
        dot(scattered.direction(), &rec.normal) > 0.
    }
}


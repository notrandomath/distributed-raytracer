use crate::raytracer::prelude::*;
use crate::raytracer::hittable::HitRecord;

#[typetag::serde(tag = "type")]
pub trait Material : Send + Sync {
    // returns true if scattered otherwise false if absorbed
    fn scatter(&self, _r_in: &Ray, _hit_record: &HitRecord, _attenuation: &mut Color, _scattered: &mut Ray) -> bool;
}

#[derive(Default, Serialize, Deserialize)]
pub struct DefaultMaterial {}

#[typetag::serde]
impl Material for DefaultMaterial {
    fn scatter(&self, _r_in: &Ray, _hit_record: &HitRecord, _attenuation: &mut Color, _scattered: &mut Ray) -> bool {
        // returns true if scattered otherwise false if absorbed
        return false;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Transparent {}

#[typetag::serde]
impl Material for Transparent {
    fn scatter(&self, r_in: &Ray, _hit_record: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        *attenuation = Color::new([1.0, 1.0, 1.0]);
        *scattered = r_in.clone();
        return true;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Lambertian {
    albedo: Color
} 

impl Lambertian {
    pub fn new(albedo: &Color) -> Self {
        Lambertian { albedo: *albedo }
    }
}

#[typetag::serde]
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

#[derive(Serialize, Deserialize)]
pub struct Metal {
    albedo: Color,
    fuzz: f64
} 

impl Metal {
    pub fn new(albedo: &Color, fuzz: f64) -> Self {
        Metal { albedo: *albedo, fuzz: if fuzz < 1.0 { fuzz } else { 1.0 } }
    }
}

#[typetag::serde]
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

#[derive(Serialize, Deserialize)]
pub struct Dialectric {
    refraction_index: f64
}

impl Dialectric {
    pub fn new(refraction_index: f64) -> Self {
        Dialectric { refraction_index }
    }

    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        // Use Schlick's approximation for reflectance.
        let mut r0 = (1. - refraction_index) / (1. + refraction_index);
        r0 = r0*r0;
        r0 + (1.-r0)*(1. - cosine).powi(5)
    }
}

#[typetag::serde]
impl Material for Dialectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, attenuation: &mut Color, scattered: &mut Ray) -> bool {
        *attenuation = Color::new([1.0, 1.0, 1.0]);
        let ri: f64 = if rec.front_face { 1.0/self.refraction_index } else { self.refraction_index };

        let unit_direction: Vec3 = unit_vector(r_in.direction());

        let cos_theta: f64 = f64::min(dot(&(-unit_direction), &rec.normal), 1.0);
        let sin_theta: f64 = (1.0 - cos_theta*cos_theta).sqrt();

        let cannot_refract: bool = ri * sin_theta > 1.0;

        let direction = if cannot_refract || Dialectric::reflectance(cos_theta, ri) > random_f64() {
            reflect(&unit_direction, &rec.normal)
        } else {
            refract(&unit_direction, &rec.normal, ri)
        };

        *scattered = Ray::new(rec.p, direction);
        return true;
    }
}
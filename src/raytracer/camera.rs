use minifb::Window;
use rand::seq::SliceRandom;
use rand::RngCore;

use crate::raytracer::prelude::*;
use crate::raytracer::hittable::{Hittable, HitRecord};

#[derive(Hash, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct PixelIndexEntry {
    pub pixel_i: i32,
    pub pixel_j: i32,
    pub pixel_sample_num: i32
} 

#[derive(Serialize, Deserialize, Clone)]
pub struct RayColorEntry {
    pub attenuation: Color,
    pub ray: Ray,
    pub depth: i32,
    pub color: Color
} 

impl RayColorEntry {
    pub fn new(ray: Ray, depth: i32) -> Self {
        RayColorEntry {
            attenuation: Color::new([1., 1., 1.]),
            ray: ray,
            depth: depth,
            color: Color::default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct RayColorStatus {
    pub finished: bool,
    pub hit_object_or_stop: bool
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,

    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,

    pub defocus_angle: f64,
    pub focus_dist: f64,

    image_height: i32,
    pub pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

pub struct CameraRayIterator<'a> {
    camera: &'a Camera,
    i: i32,
    n: i32,
    shuffled_seq: Vec<i32>
}

impl<'a> CameraRayIterator<'a> {
    fn new(camera: &'a Camera) -> Self {
        let n = camera.image_width * camera.image_height;
        let shuffled = (0..n).collect::<Vec<i32>>();

        CameraRayIterator {
            camera: camera,
            i: 0,
            n: n,
            shuffled_seq: shuffled
        }
    }
}

impl<'a> Iterator for CameraRayIterator<'a> {
    type Item = (PixelIndexEntry, Ray);

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n {
            return None;
        }
        if self.i % self.n == 0 {
            self.shuffled_seq.shuffle(&mut rand::rng());
        }
        if self.i % self.camera.image_width == 0 {
            println!("line {} / {} (sample {})", (self.i / self.camera.image_width) % self.camera.image_height, self.camera.image_height, self.i / self.n);
        }

        let cur = self.shuffled_seq[(self.i % self.n) as usize];
        let pixel_j = cur / self.camera.image_width;
        let pixel_i = cur % self.camera.image_width;
        let pixel_sample_num = self.i / self.n;
        let ray = self.camera.get_ray(pixel_i, pixel_j);
        
        self.i += 1;
        Some((PixelIndexEntry {
            pixel_i: pixel_i,
            pixel_j: pixel_j,
            pixel_sample_num: pixel_sample_num,
        }, ray))
    }
}

pub fn ray_color_iteration(r: &mut RayColorEntry, world: &dyn Hittable) -> RayColorStatus {
    // performs a single iteration of ray_color
    if r.depth <= 0 {
        r.color = r.attenuation*Color::new([0.,0.,0.]);
        return RayColorStatus{finished: true, hit_object_or_stop: true};
    }

    let mut rec: HitRecord = HitRecord::default();
    if world.hit(&r.ray, Interval::new_min_max(0.001, INFINITY), &mut rec) {
        let mut scattered: Ray = Ray::default();
        let mut attenuation: Color = Color::default();
        if rec.mat.scatter(&r.ray, &rec, &mut attenuation, &mut scattered) {
            r.attenuation = r.attenuation * attenuation;
            r.ray = scattered;
            r.depth -= 1;
            return RayColorStatus{finished: false, hit_object_or_stop: true};
        } else {
            r.color = r.attenuation*Color::new([0.,0.,0.]);
            return RayColorStatus{finished: true, hit_object_or_stop: true};
        }
    }

    let unit_direction: Vec3 = unit_vector(r.ray.direction());
    let a = 0.5*(unit_direction.y() + 1.0);
    r.color = r.attenuation *((1.0-a)*Color::new([1.0, 1.0, 1.0]) + a*Color::new([0.5, 0.7, 1.0]));
    return RayColorStatus{finished: true, hit_object_or_stop: false};
}

impl Camera {
    pub fn new() -> Self {
        let mut camera: Camera = Camera::default();
        camera.aspect_ratio = 1.0;
        camera.image_width = 100;
        camera.samples_per_pixel = 10;
        camera.max_depth = 10;

        camera.vfov = 90.;
        camera.lookfrom = Point3::new([0., 0., 0.]);
        camera.lookat = Point3::new([0., 0., -1.]);
        camera.vup = Vec3::new([0., 1., 0.]);

        camera.defocus_angle = 0.;
        camera.focus_dist = 10.;

        camera
    }

    pub fn iterate_rays(&self) -> CameraRayIterator {
        CameraRayIterator::new(&self)
    }

    pub fn initialize(&mut self) {
        // Calculate the image height, and ensure that it's at least 1.
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as i32;
        self.image_height = if self.image_height < 1 { 1 } else { self.image_height };

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.lookfrom;


        // Determine viewport dimensions.
        let theta = degrees_to_radians(self.vfov);
        let h = f64::tan(theta/2.);
        let viewport_height =  2. * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // Calculate the u,v,w unit basis vectors for the camera coordinate frame.
        self.w = unit_vector(&(self.lookfrom - self.lookat));
        self.u = unit_vector(&cross(&self.vup, &self.w));
        self.v = cross(&self.w, &self.u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u: Vec3 = viewport_width * self.u;
        let viewport_v: Vec3 = viewport_height * -self.v;

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = self.center - (self.focus_dist * self.w) - viewport_u/2. - viewport_v/2.;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        // Calculate the camera defocus disk basis vectors.
        let defocus_radius = self.focus_dist * f64::tan(degrees_to_radians(self.defocus_angle / 2.));
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    pub fn render(
        &mut self, world: &dyn Hittable, 
        window: &mut Window, 
        color_buffer: &mut Vec<u32>,
        raw_buffer: &mut Vec<Vec3>,
        count_buffer: &mut Vec<i32>
    ) -> Result<()> {
        self.initialize();

        for sample in 0..self.samples_per_pixel {
            for j in 0..self.image_height {
                for i in 0..self.image_width {
                    if i == 0 {
                        println!("line {} / {} (sample {})", j, self.image_height, sample);
                    }
                    let mut pixel_color: Color = Color::new_xyz(0.,0.,0.);
                    let r: Ray = self.get_ray(i, j);
                    pixel_color += self.ray_color(&r, self.max_depth, world);
                    write_color(
                        i, j, 
                        self.image_width as usize, 
                        self.image_height as usize, 
                        &pixel_color, window, color_buffer, raw_buffer, count_buffer
                    )?;
                }
            }
        }
        Ok(())
    }

    fn get_ray(&self, i: i32, j: i32) -> Ray {
        // Construct a camera ray originating from the defocus disk and directed at a randomly
        // sampled point around the pixel location i, j.

        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc
                          + ((i as f64 + offset.x()) * self.pixel_delta_u)
                          + ((j as f64 + offset.y()) * self.pixel_delta_v);

        let ray_origin =  if self.defocus_angle <= 0. { self.center } else { self.defocus_disk_sample() };
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn sample_square(&self) -> Vec3 {
        // Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
        return Vec3::new_xyz(random_f64() - 0.5, random_f64() - 0.5, 0.);
    }

    fn defocus_disk_sample(&self) -> Vec3 {
        // Returns a random point in the camera defocus disk.
        let p = random_in_unit_disk();
        return self.center + (p[0] * self.defocus_disk_u) + (p[1] * self.defocus_disk_v);
    }

    fn ray_color(&self, r: &Ray, depth: i32, world: &dyn Hittable) -> Color {
        if depth <= 0 {
            return Color::new([0.,0.,0.]);
        }

        let mut rec: HitRecord = HitRecord::default();
        if world.hit(r, Interval::new_min_max(0.001, INFINITY), &mut rec) {
            let mut scattered: Ray = Ray::default();
            let mut attenuation: Color = Color::default();
            if rec.mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                return attenuation * self.ray_color(&scattered, depth-1, world);
            }
            return Color::new([0.,0.,0.]);
        }

        let unit_direction: Vec3 = unit_vector(r.direction());
        let a = 0.5*(unit_direction.y() + 1.0);
        (1.0-a)*Color::new([1.0, 1.0, 1.0]) + a*Color::new([0.5, 0.7, 1.0])
    }
}
use crate::prelude::*;
use crate::hittable::{Hittable, HitRecord};

#[derive(Default)]
pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: i32,
    pub samples_per_pixel: i32,
    image_height: i32,
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3
}

impl Camera {
    pub fn new() -> Self {
        let mut camera: Camera = Camera::default();
        camera.aspect_ratio = 1.0;
        camera.image_width = 100;
        camera.samples_per_pixel = 10;
        camera
    }

    fn initialize(&mut self) {
        // Calculate the image height, and ensure that it's at least 1.
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as i32;
        self.image_height = if self.image_height < 1 { 1 } else { self.image_height };

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = Point3::new([0., 0., 0.]);

        // Determine viewport dimensions.
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // Calculate the vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = Vec3::new([viewport_width, 0., 0.]);
        let viewport_v = Vec3::new([0., -viewport_height, 0.]);

        // Calculate the horizontal and vertical delta vectors from pixel to pixel.
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // Calculate the location of the upper left pixel.
        let viewport_upper_left = self.center
                                - Vec3::new([0., 0., focal_length])
                                - viewport_u/2. - viewport_v/2.;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);
    }

    pub fn render(&mut self, world: &dyn Hittable, writer: &mut BufWriter<File>) -> Result<()> {
        self.initialize();

        let mut err = stderr();
        write!(writer, "P3\n{} {}\n255\n", self.image_width, self.image_height)?;
        for j in 0..self.image_height {
            write!(err, "\rScanlines remaining: {} ", self.image_height-j)?;
            err.flush()?;
            for i in 0..self.image_width {
                let mut pixel_color: Color = Color::new_xyz(0.,0.,0.);
                for _sample in 0..self.samples_per_pixel {
                    let r: Ray = self.get_ray(i, j);
                    pixel_color += self.ray_color(&r, world);
                }
                write_color(writer, &(self.pixel_samples_scale * pixel_color))?;
            }
        }
        write!(err, "\rDone.                                  \n")?;

        Ok(())
    }

    fn get_ray(&self, i: i32, j: i32) -> Ray {
        // Construct a camera ray originating from the origin and directed at randomly sampled
        // point around the pixel location i, j.

        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc
                          + ((i as f64 + offset.x()) * self.pixel_delta_u)
                          + ((j as f64 + offset.y()) * self.pixel_delta_v);

        let ray_origin = self.center;
        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    fn sample_square(&self) -> Vec3 {
        // Returns the vector to a random point in the [-.5,-.5]-[+.5,+.5] unit square.
        return Vec3::new_xyz(random_f64() - 0.5, random_f64() - 0.5, 0.);
    }

    fn ray_color(&self, r: &Ray, world: &dyn Hittable) -> Color {
        let mut rec: HitRecord = HitRecord::default();
        if world.hit(r, 0., INFINITY, &mut rec) {
            return 0.5 * (rec.normal + Color::new_xyz(1.,1.,1.));
        }

        let unit_direction: Vec3 = unit_vector(r.direction());
        let a = 0.5*(unit_direction.y() + 1.0);
        (1.0-a)*Color::new([1.0, 1.0, 1.0]) + a*Color::new([0.5, 0.7, 1.0])
    }
}
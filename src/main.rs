mod vec3;
mod colors;
mod ray;
mod hittable;
mod hittable_list;
mod sphere;
mod prelude;
mod interval;
mod camera;

use crate::prelude::*;
use crate::sphere::Sphere;
use crate::hittable_list::HittableList;
use crate::camera::Camera;

const OUTPUT_FILENAME: &str = "img.ppm";

fn main() -> Result<()>  {
    let mut world: HittableList = HittableList::new();
    world.add(Rc::new(Sphere::new(&Point3::new_xyz(0.,0.,-1.), 0.5)));
    world.add(Rc::new(Sphere::new(&Point3::new_xyz(0.,-100.5,-1.), 100.)));

    let mut writer = BufWriter::new(File::create(OUTPUT_FILENAME)?);
    let mut camera: Camera = Camera::new();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;

    camera.render(&world, &mut writer)?;

    Ok(())
}
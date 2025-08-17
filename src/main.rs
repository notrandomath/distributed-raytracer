mod vec3;
mod colors;
mod ray;
mod hittable;
mod hittable_list;
mod sphere;
mod prelude;
mod interval;
mod camera;
mod material;

use rand::rand_core::le;

use crate::prelude::*;
use crate::sphere::Sphere;
use crate::hittable_list::HittableList;
use crate::camera::Camera;
use crate::material::*;

const OUTPUT_FILENAME: &str = "img.ppm";

fn main() -> Result<()>  {
    let mut world: HittableList = HittableList::new();

    let material_ground: Rc<dyn Material> = Rc::new(Lambertian::new(&Color::new_xyz(0.8, 0.8, 0.0)));
    let material_center: Rc<dyn Material> = Rc::new(Lambertian::new(&Color::new_xyz(0.1, 0.2, 0.5)));
    let material_left: Rc<dyn Material> = Rc::new(Metal::new(&Color::new_xyz(0.8, 0.8, 0.8), 0.3));
    let material_right: Rc<dyn Material> = Rc::new(Metal::new(&Color::new_xyz(0.8, 0.6, 0.2), 1.0));

    world.add(Rc::new(Sphere::new(&Point3::new_xyz(0.,-100.5,-1.), 100., material_ground)));
    world.add(Rc::new(Sphere::new(&Point3::new_xyz(0.,0.,-1.2), 0.5, material_center)));
    world.add(Rc::new(Sphere::new(&Point3::new_xyz(-1.,0.,-1.), 0.5, material_left)));
    world.add(Rc::new(Sphere::new(&Point3::new_xyz(1.,0.,-1.), 0.5, material_right)));

    let mut writer = BufWriter::new(File::create(OUTPUT_FILENAME)?);
    let mut camera: Camera = Camera::new();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 400;
    camera.samples_per_pixel = 100;
    camera.max_depth = 50;

    camera.render(&world, &mut writer)?;

    Ok(())
}
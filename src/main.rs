use dray_lib::raytracer::prelude::*;
use dray_lib::raytracer::sphere::Sphere;
use dray_lib::raytracer::hittable_list::HittableList;
use dray_lib::raytracer::camera::Camera;
use dray_lib::raytracer::material::*;
use minifb::{Window, WindowOptions};

const OUTPUT_FILENAME: &str = "img.ppm";

fn main() -> Result<()>  {
    let mut world: HittableList = HittableList::new();

    let ground_material = Arc::new(Lambertian::new(&Color::new([0.5, 0.5, 0.5])));
    world.add(Arc::new(Sphere::new(&Point3::new_xyz(0.,-1000.,0.), 1000., ground_material)));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_f64();
            let center: Point3 = Point3::new_xyz((a as f64) + 0.9*random_f64(), 0.2, (b as f64) + 0.9*random_f64());

            if (center - Point3::new_xyz(4., 0.2, 0.)).length() > 0.9 {
                let mut sphere_material: Arc<dyn Material> = Arc::new(DefaultMaterial::default());

                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random() * Color::random();
                    sphere_material = Arc::new(Lambertian::new(&albedo));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random_range(0.5, 1.);
                    let fuzz = random_f64_range(0., 0.5);
                    sphere_material = Arc::new(Metal::new(&albedo, fuzz));
                } else {
                    // glass
                    sphere_material = Arc::new(Dialectric::new(1.5));
                }

                world.add(Arc::new(Sphere::new(&center, 0.2, sphere_material)));
            }
        }
    }

    let material1 = Arc::new(Dialectric::new(1.5));
    world.add(Arc::new(Sphere::new(&Point3::new_xyz(0., 1., 0.), 1.0, material1)));

    let material2 = Arc::new(Lambertian::new(&Color::new([0.4, 0.2, 0.1])));
    world.add(Arc::new(Sphere::new(&Point3::new_xyz(-4., 1., 0.), 1.0, material2)));

    let material3 = Arc::new(Metal::new(&Color::new([0.7, 0.6, 0.5]), 0.0));
    world.add(Arc::new(Sphere::new(&Point3::new_xyz(4., 1., 0.), 1.0, material3)));

    let mut writer = BufWriter::new(File::create(OUTPUT_FILENAME)?);
    let mut camera: Camera = Camera::new();

    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = 1200;
    camera.samples_per_pixel = 500;
    camera.max_depth = 50;

    camera.vfov     = 20.;
    camera.lookfrom = Point3::new_xyz(13.,2.,3.);
    camera.lookat   = Point3::new_xyz(0.,0.,0.);
    camera.vup      = Vec3::new_xyz(0.,1.,0.);

    camera.defocus_angle = 0.6;
    camera.focus_dist    = 10.0;

    // Initialize Image Buffer
    let width = camera.image_width as usize;
    let height = camera.image_width as usize;
    let mut color_buffer: Vec<u32> = vec![0; width * height];
    let mut raw_buffer: Vec<Vec3> = vec![Vec3::new([0., 0., 0.]); width * height];
    let mut count_buffer: Vec<i32> = vec![0; width * height];

    // Create the window.
    let mut window = Window::new(
        "Raytracer Image (normal)",
        width,
        height,
        WindowOptions::default(),
    ).unwrap();
    
    // Set a frame rate limit for efficiency.
    window.set_target_fps(60);

    camera.render(&world, &mut window, &mut color_buffer, &mut raw_buffer, &mut count_buffer)?;

    Ok(())
}
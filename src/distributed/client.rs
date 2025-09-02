use std::io::Result;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};
use futures_util::stream::{SplitSink, StreamExt};

use crate::distributed::config::ORCHESTRATOR_CLIENT_CONNECTION_SOCKET;
use crate::distributed::distributed_common::send_websocket_message;
use crate::distributed::messages::OrchestratorServerMessage;
use crate::raytracer::camera::Camera;
use crate::raytracer::colors::color_to_rgb;
use crate::raytracer::material::*;
use crate::raytracer::prelude::*;
use crate::raytracer::sphere::Sphere;

use minifb::{Window, WindowOptions};


async fn send_objects(
    write: &mut SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
) {
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

                send_websocket_message(
                    write,
                    &OrchestratorServerMessage::new_add_object(Arc::new(Sphere::new(&center, 0.2, sphere_material)))
                ).await.unwrap();
            };
        }
    }
}

pub async fn run_client() -> Result<()> {
    // Initialize Camera
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

    camera.initialize();

    // Initialize Image Buffer
    let width = camera.image_width as usize;
    let height = camera.image_width as usize;
    let mut color_buffer: Vec<u32> = vec![0; width * height];
    let mut raw_buffer: Vec<Vec3> = vec![Vec3::new([0., 0., 0.]); width * height];
    let mut count_buffer: Vec<i32> = vec![0; width * height];

    // Create the window.
    let mut window = Window::new(
        "Raytracer Image (distributed)",
        width,
        height,
        WindowOptions::default(),
    ).unwrap();
    
    // Set a frame rate limit for efficiency.
    window.set_target_fps(60);

    // Connect to a local WebSocket server.
    let addr = ORCHESTRATOR_CLIENT_CONNECTION_SOCKET;
    let url = format!("ws://{}:{}", addr.ip(), addr.port());
    let (ws_stream, _) = connect_async(url).await.unwrap();
    println!("WebSocket handshake with localhost successful!");

    println!("Sending objects...");
    let (mut write, mut read) = ws_stream.split();
    send_objects(&mut write).await;

    println!("Starting raytracing...");
    send_websocket_message(&mut write, &OrchestratorServerMessage::new_raytrace(&camera)).await.unwrap();

    println!("Awaiting rays...");
    window.update_with_buffer(&color_buffer, width, height).unwrap();
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(_)) => {}
            Ok(Message::Binary(binary)) => {
                let (msg, _num_bytes_decoded): (OrchestratorServerMessage, usize) = bincode::serde::decode_from_slice(
                    &binary, bincode::config::standard()).unwrap();
                let pixel_idx = msg.pixel_index.unwrap();
                let index = pixel_idx.pixel_j as usize * width + pixel_idx.pixel_i as usize;
                let pixel_color = msg.pixel_color.unwrap();
                raw_buffer[index] += pixel_color;
                count_buffer[index] += 1;
                let denom = if count_buffer[index] != 0 {count_buffer[index] as f64} else {1.};
                let (rbyte, gbyte, bbyte) = color_to_rgb(&(raw_buffer[index] / denom));
                let color: u32 = (255 << 24) | (rbyte << 16) | (gbyte << 8) | bbyte;
                color_buffer[index] = color;

                window.update_with_buffer(&color_buffer, width, height).unwrap();
            }
            Ok(Message::Ping(_)) => {}
            Ok(Message::Close(_)) => {}
            Ok(Message::Pong(_)) => {}
            Ok(Message::Frame(_)) => {}
            Err(_) => {}
        }
    }
    Ok(())
}
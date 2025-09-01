use std::net::{SocketAddrV4};
use std::io::{Result};
use std::error::Error;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};
use tokio::net::TcpStream;
use futures_util::stream::{SplitSink, StreamExt};
use futures_util::sink::SinkExt;

use crate::distributed::messages::{OrchestratorServerMessage, OrchestratorServerMessageType};
use crate::raytracer::material::*;
use crate::raytracer::prelude::*;
use crate::raytracer::sphere::Sphere;

pub struct Client {
    
}

pub async fn send_tcp_message<T: Serialize>(
    write: &mut SplitSink<WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>,
    message: &T,
) -> Result<()> { 
    // Encode data
    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&message, bincode::config::standard()).unwrap();

    // Write all bytes to the stream
    write
        .send(tokio_tungstenite::tungstenite::Message::binary(message_bytes))
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}

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

                send_tcp_message(
                    write,
                    &OrchestratorServerMessage::new_add_object(Arc::new(Sphere::new(&center, 0.2, sphere_material)))
                ).await.unwrap();
            };
        }
    }
}

pub async fn run_client(addr: SocketAddrV4) -> Result<()> {
    // Connect to a local WebSocket server.
    let url = format!("ws://{}:{}", addr.ip(), addr.port());
    let (ws_stream, _) = connect_async(url).await.unwrap();
    println!("WebSocket handshake with localhost successful!");

    let (mut write, mut read) = ws_stream.split();

    send_objects(&mut write).await;

    send_tcp_message(&mut write, &OrchestratorServerMessage::new_no_data(
        OrchestratorServerMessageType::BeginRaytracing)).await.unwrap();

    loop {
        // Read the response from the local server.
        if let Some(msg) = read.next().await {
            let msg = msg.unwrap();
            println!("Received: {}", msg);
        }
    }
}
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream, UdpSocket};
use std::time::Duration;
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT};
use crate::distributed::messages::{ObjectServerMessage, ObjectServerMessageType, ServerDiscoveryMessage, ServerType, NUM_SERVER_TYPES};
use crate::raytracer::bounding_box::BoundingBox;
use crate::raytracer::hittable_list::HittableList;
use crate::raytracer::material::*;
use crate::raytracer::prelude::*;
use crate::raytracer::sphere::Sphere;
use std::io::{ErrorKind, Read, Result, Write};
use std::collections::{HashMap};

pub struct Client {
    server_directory: [Vec<SocketAddrV4>; NUM_SERVER_TYPES],
    boxes: Vec<Rc<BoundingBox>>,
    boxes_hittable: HittableList,
    object_map: HashMap<usize, SocketAddrV4>,
}

pub fn send_tcp_message<T: Serialize>(socket_addr: &SocketAddrV4, message: &T) -> Result<String> {    
    // 1. Establish the connection
    let mut stream = match TcpStream::connect(socket_addr) {
        Ok(s) => {
            s
        },
        Err(e) => {
            eprintln!("🛑 Failed to connect: {}", e);
            return Err(e); // Exit the function with the connection error
        }
    };

    // 2. Write data to the server
    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&message, 
        bincode::config::standard()).unwrap();
    
    // Write all bytes to the stream
    stream.write_all(message_bytes.as_slice())?;
    
    // 3. Read the server's response (e.g., an echo)
    let mut buffer = [0; 128]; // Buffer to hold the response
    
    // Read the response into the buffer
    let bytes_read = stream.read(&mut buffer)?;
    
    // Convert the received bytes into a readable string
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);

    Ok(response.into_owned())
}

impl Client {
    pub fn new() -> Self {
        Client {
            server_directory: std::array::from_fn(|_| Vec::new()),
            boxes: Vec::new(),
            boxes_hittable: HittableList::new(),
            object_map: HashMap::new(),
        }
    }

    fn create_bounding_volumes(&mut self) {
        let n = self.server_directory[ServerType::Object as usize].len();
        let mut cur_i: usize = 0;
        for a in (-10..=10).step_by(4) {
            for b in (-10..=10).step_by(4) {
                let bv = BoundingBox::new_xyz(
                    if a == -10 {-1e6} else {(a as f64) - 4.},
                    if a == 10 {1e6} else {(a as f64) + 4.},
                    -1e6,
                    1e6,
                    if b == -10 {-1e6} else {(b as f64) - 4.},
                    if b == 10 {1e6} else {(b as f64) + 4.},
                );
                self.object_map.insert(self.boxes.len(), self.server_directory[ServerType::Object as usize][cur_i]);
                let new_box = Rc::new(bv);
                self.boxes.push(new_box.clone());
                self.boxes_hittable.add(new_box.clone());
                cur_i = (cur_i+1)%n;
            }
        }
    }

    fn create_objects(&self) {
        for a in -11..11 {
            for b in -11..11 {
                let center: Point3 = Point3::new_xyz((a as f64) + 0.9*random_f64(), 0.2, (b as f64) + 0.9*random_f64());
                if (center - Point3::new_xyz(4., 0.2, 0.)).length() > 0.9 {
                    let choose_mat = random_f64();
                    let mut sphere_material: Rc<dyn Material> = Rc::new(DefaultMaterial::default());

                    if choose_mat < 0.8 {
                        // diffuse
                        let albedo = Color::random() * Color::random();
                        sphere_material = Rc::new(Lambertian::new(&albedo));
                    } else if choose_mat < 0.95 {
                        // metal
                        let albedo = Color::random_range(0.5, 1.);
                        let fuzz = random_f64_range(0., 0.5);
                        sphere_material = Rc::new(Metal::new(&albedo, fuzz));
                    } else {
                        // glass
                        sphere_material = Rc::new(Dialectric::new(1.5));
                    }

                    let new_sphere = Rc::new(Sphere::new(&center, 0.2, sphere_material));
                    for (index, aabb) in self.boxes.iter().enumerate() {
                        if aabb.intersect_sphere(&new_sphere) {
                            let _ = send_tcp_message(
                                &self.object_map[&index], 
                                &ObjectServerMessage::new_object_add(new_sphere.clone())
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Bind to the socket that will receive the multicast packets
        let socket = UdpSocket::bind(SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT))?;
        
        // Join the multicast group on the local interface (0.0.0.0)
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?;
        socket.set_read_timeout(Some(Duration::from_secs(10)))?;
        
        println!("Joined multicast group and listening for messages...");                 

        let mut buf = [0; 256];
        loop {
            // Receive data from the socket
            let recv_result = socket.recv_from(&mut buf);

            match recv_result {
                Ok((num_bytes, _src_addr)) => {
                    let (msg, _num_bytes_decoded): (ServerDiscoveryMessage, usize) = bincode::serde::decode_from_slice(
                        &buf[..num_bytes], bincode::config::standard()).unwrap();
                    if !self.server_directory[msg.server_type as usize].contains(&msg.socket_addr) {
                        self.server_directory[msg.server_type as usize].push(msg.socket_addr);
                        send_tcp_message(&msg.socket_addr, &ObjectServerMessage::new_no_data(ObjectServerMessageType::Deregistration))?;
                    }
                }
                // Error: Check if it's a timeout error
                Err(ref e) if e.kind() == ErrorKind::TimedOut || e.kind() == ErrorKind::WouldBlock => {
                    println!("\nTimeout reached.");
                    break;
                },
                // Other error
                Err(e) => {
                    return Err(e.into());
                }
            }
        }

        println!("\nFinal Server Directory:");
        for (i, set) in self.server_directory.iter().enumerate() {
            println!("Type {}: {} servers found", i, set.len());
            for addr in set.iter() {
                println!("  - {}", addr);
            }
        }

        println!("Adding objects...");
        self.create_bounding_volumes();
        self.create_objects();

        println!("Printing objects...");
        for addr in self.server_directory[ServerType::Object as usize].iter() {
            let _result = send_tcp_message(
                addr, 
                &ObjectServerMessage::new_no_data(ObjectServerMessageType::PrintObjects)
            );
        }

        Ok(())
    }
}

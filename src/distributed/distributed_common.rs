use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::{atomic::{AtomicBool, Ordering}, Arc};
use std::thread;
use std::time::Duration;
use std::io::{Result};
use futures_util::stream::SplitSink;
use futures_util::sink::SinkExt;
use serde::de::DeserializeOwned;
use serde::{Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::WebSocketStream;
use crate::distributed::config::{MULTICAST_ADDR, MULTICAST_PORT};
use crate::distributed::messages::{ObjectServerMessage, RayServerMessage, ServerDiscoveryMessage, ServerType};
use crate::distributed::ray_server::{RayServer};
use crate::distributed::{object_server::ObjectServer};

pub async fn run_async_server<M: Serialize + DeserializeOwned + Clone, F, U>(socket_addr: SocketAddrV4, mut handler: F) -> Result<()> 
where 
    F: FnMut(&mut M) -> U
{
    let listener  = TcpListener::bind(socket_addr).await?;
    while let Ok((mut stream, _)) = listener.accept().await {
        // Read the length
        let mut len_bytes = [0; 4];
        stream.read_exact(&mut len_bytes).await?;
        let message_len = u32::from_le_bytes(len_bytes) as usize;
        // Read the message
        let mut buf = vec![0; message_len];
        stream.read_exact(&mut buf).await?;
        // Convert the bytes into a decoded server message
        let (mut msg, _num_bytes_decoded): (M, usize) = bincode::serde::decode_from_slice(
            &buf, bincode::config::standard()).unwrap();
        handler(&mut msg);
        // Writes new message (msg was modified by self.handle_msg)
        let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&msg, 
            bincode::config::standard()).unwrap();
        stream.write_all(message_bytes.as_slice()).await?;
    }
    Ok(())
}

pub async fn send_websocket_message<T: Serialize, S: AsyncRead + AsyncWrite + Unpin>(
    write: &mut SplitSink<WebSocketStream<S>, Message>,
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

pub async fn send_tcp_message<T: Serialize>(socket_addr: &SocketAddrV4, message: &T) -> Result<Vec<u8>> {    
    // 1. Establish the connection
    let timeout_duration = Duration::from_secs(5);
    let mut stream = tokio::time::timeout(timeout_duration, TcpStream::connect(socket_addr)).await??;

    // 2. Write data to the server
    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&message, 
        bincode::config::standard()).unwrap();
    
    // Write all bytes to the stream
    tokio::time::timeout(timeout_duration,stream.write_all(&(message_bytes.len() as u32).to_le_bytes())).await??;
    tokio::time::timeout(timeout_duration,stream.write_all(message_bytes.as_slice())).await??;
    
    // 3. Read the server's response (e.g., an echo)
    let mut buffer = [0; 128]; // Buffer to hold the response
    
    // Read the response into the buffer
    let bytes_read = tokio::time::timeout(timeout_duration,stream.read(&mut buffer)).await??;
    
    // Convert the received bytes into a readable string
    let response = buffer[..bytes_read].to_vec();

    Ok(response)
}

// This function multicasts the server's port number
async fn multicast_port_announcer(
    port_to_announce: u16,
    server_type: ServerType, 
    should_stop: Arc<AtomicBool>
) -> Result<()> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)).await?;
    socket.set_multicast_loop_v4(true)?; // Since server and client are both on localhost
    
    let multicast_addr = SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT);

    println!("Multicasting port {} to {}", port_to_announce, multicast_addr);

    let message = ServerDiscoveryMessage{
        server_type: server_type,
        socket_addr: SocketAddrV4::new(Ipv4Addr::LOCALHOST, port_to_announce)
    };
    
    let message_bytes: Vec<u8> = bincode::serde::encode_to_vec(&message, 
        bincode::config::standard()).unwrap();

    loop {
        if should_stop.load(Ordering::SeqCst) {
            tokio::time::sleep(Duration::from_secs(3)).await; // Check every 3 seconds
        } else {
            socket.send_to(&message_bytes, multicast_addr).await?;
            tokio::time::sleep(Duration::from_secs(3)).await; // Announce every 3 seconds
        }
    }
}

pub async fn run_server(port: u16, is_object_server: bool) {
    let should_stop = Arc::new(AtomicBool::new(false));
    let multicast_stop_flag = Arc::clone(&should_stop);

    // Start the multicast announcer in a separate thread.
    let multicast_handle = tokio::spawn( 
        multicast_port_announcer(port, 
            if is_object_server {ServerType::Object} else {ServerType::Ray},
            multicast_stop_flag
        )
    );

    // Start the TCP server in a separate thread.
    let socket_addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    if is_object_server {
        let _ = tokio::spawn(async move {
            let mut server = ObjectServer::new(Arc::clone(&should_stop));
            run_async_server::<ObjectServerMessage, _, ()>(
                socket_addr, 
                move |msg| {
                    server.handle_msg(msg);
                }
            ).await
        }).await;
    } else {
        let _ = tokio::spawn(async move {
            let mut server = RayServer::new(Arc::clone(&should_stop));
            run_async_server::<RayServerMessage, _, ()>(
                socket_addr, 
                move |msg| {
                    server.handle_msg(msg);
                }
            ).await
        }).await;
    }

    // Wait for both threads to finish (which they won't, as they run infinitely).
    let _ = multicast_handle.await;
}
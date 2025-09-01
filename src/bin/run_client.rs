use std::net::SocketAddrV4;

use dray_lib::distributed::client::{run_client};
use dray_lib::distributed::config::{ORCHESTRATOR_PORT};

#[tokio::main]
async fn main() {
    let _ = run_client(SocketAddrV4::new(std::net::Ipv4Addr::LOCALHOST, ORCHESTRATOR_PORT)).await;
}
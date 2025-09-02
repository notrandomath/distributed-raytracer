use dray_lib::distributed::client::{run_client};

#[tokio::main]
async fn main() {
    let _ = run_client().await;
}
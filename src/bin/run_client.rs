use dray_lib::distributed::client::Client;

fn main() {
    let mut client = Client::new();
    let _ = client.run();
}
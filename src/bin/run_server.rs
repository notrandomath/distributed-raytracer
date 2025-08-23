use dray_lib::distributed::server::run_server;
use std::thread;

fn main() {
    let mut handles = vec![];

    for i in 0..3 {
        let handle = thread::spawn(move || {
            println!("Starting server thread #{}", i);
            run_server();
        });
        handles.push(handle);
    }

    // Wait for all server threads to finish
    for handle in handles {
        handle.join().unwrap();
    }
}
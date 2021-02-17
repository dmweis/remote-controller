use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let controller_state =
        remote_controller::start_remote_controller_server(([127, 0, 0, 1], 8080));

    loop {
        sleep(Duration::from_millis(20)).await;
        println!("State: {:?}", controller_state.lock().unwrap().get_latest());
    }
}

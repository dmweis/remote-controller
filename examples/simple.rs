use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let mut state_handle = remote_controller::start_remote_controller_server(([0, 0, 0, 0], 8080));

    loop {
        sleep(Duration::from_millis(20)).await;
        println!("gamepad: {:?}", state_handle.get_last_gamepad_command());
        println!("touch: {:?}", state_handle.get_latest_canvas_touch());
    }
}

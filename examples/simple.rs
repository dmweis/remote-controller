use remote_controller::{Action, ActionList, AreaSize};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let action_list = ActionList::new(vec![
        Action::new(String::from("save"), String::from("Save current position")),
        Action::new(String::from("load"), String::from("Load current position")),
    ]);

    let mut state_handle = remote_controller::start_remote_controller_server_with_map(
        ([0, 0, 0, 0], 8080),
        AreaSize::new(1.0, 2.0),
        action_list,
    );

    loop {
        sleep(Duration::from_millis(20)).await;
        println!("gamepad: {:?}", state_handle.get_last_gamepad_command());
        println!("touch: {:?}", state_handle.get_latest_canvas_touch());
        if let Some(message) = state_handle.check_new_actions().unwrap() {
            println!("Action received {}", message);
        }
    }
}

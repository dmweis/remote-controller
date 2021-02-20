use futures::{SinkExt, StreamExt};
use log::*;
use serde::Deserialize;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::{sleep, timeout};
use warp::{
    ws::{Message, WebSocket},
    Filter,
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GamepadCommand {
    #[serde(rename = "lx")]
    pub left_x: f32,
    #[serde(rename = "ly")]
    pub left_y: f32,
    #[serde(rename = "rx")]
    pub right_x: f32,
    #[serde(rename = "ry")]
    pub right_y: f32,
}

#[derive(Default)]
pub struct ControllerState {
    last_command: Mutex<GamepadCommand>,
}

impl ControllerState {
    fn update(&self, command: GamepadCommand) {
        let mut guard = self.last_command.lock().unwrap();
        *guard = command;
    }

    pub fn get_latest(&mut self) -> GamepadCommand {
        self.last_command.lock().unwrap().clone()
    }
}

pub type SharedControllerState = Arc<Mutex<ControllerState>>;

async fn handle_websocket(ws: WebSocket, controller_state: SharedControllerState) {
    trace!("new websocket connection");
    let (mut ws_tx, mut ws_rx) = ws.split();

    tokio::task::spawn(async move {
        loop {
            sleep(HEARTBEAT_INTERVAL).await;
            if ws_tx.send(Message::ping("")).await.is_err() {
                error!("Failed to send ping");
                break;
            }
        }
    });

    while let Ok(Some(result)) = timeout(CLIENT_TIMEOUT, ws_rx.next()).await {
        match result {
            Ok(msg) => {
                if let Ok(text) = msg.to_str() {
                    if let Ok(command) = serde_json::from_str(text) {
                        controller_state.lock().unwrap().update(command);
                    } else {
                        error!("Failed to parse json {}", text);
                    }
                } else if msg.is_pong() {
                    trace!("Pong received");
                } else if msg.is_close() {
                    trace!("Got closing message. Closing connection");
                } else {
                    error!("Unknown message type {:?}", msg);
                }
            }
            Err(e) => {
                error!("websocket error: {}", e);
                break;
            }
        }
    }

    error!("User connection ended");
}

pub fn start_remote_controller_server(
    address: impl Into<std::net::SocketAddr>,
) -> SharedControllerState {
    let address = address.into();
    let controller_state = SharedControllerState::default();
    let controller_state_clone = Arc::clone(&controller_state);
    let shared_controller_state = warp::any().map(move || controller_state_clone.clone());
    let ws = warp::path("ws")
        .and(warp::ws())
        .and(shared_controller_state)
        .map(|ws: warp::ws::Ws, controller| {
            ws.on_upgrade(move |socket| handle_websocket(socket, controller))
        });

    // manually construct paths since this allows us to embed the files into the binary
    let index = warp::path::end().map(|| warp::reply::html(include_str!("../static/index.html")));

    let gamepad = warp::path::path("gamepad.js")
        .map(|| warp::reply::html(include_str!("../static/gamepad.js")));
    let controller = warp::path::path("controller.js")
        .map(|| warp::reply::html(include_str!("../static/controller.js")));
    let nipplejs = warp::path::path("nipplejs.js")
        .map(|| warp::reply::html(include_str!("../static/nipplejs.js")));
    let style = warp::path::path("style.css").map(|| {
        warp::reply::with_header(
            include_str!("../static/style.css"),
            "Content-Type",
            "text/css",
        )
    });
    let touch_control = warp::path::path("touchControl.js")
        .map(|| warp::reply::html(include_str!("../static/touchControl.js")));

    let routes = index
        .or(ws)
        .or(gamepad)
        .or(controller)
        .or(nipplejs)
        .or(style)
        .or(touch_control);

    tokio::task::spawn(async move {
        warp::serve(routes).run(address).await;
    });

    controller_state
}

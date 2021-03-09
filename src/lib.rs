use futures::{SinkExt, StreamExt};
use include_dir::{include_dir, Dir};
use log::*;
use serde::{Deserialize, Serialize};
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
pub struct CanvasTouch {
    pub width: f32,
    pub height: f32,
    pub down_x: f32,
    pub down_y: f32,
    pub up_x: f32,
    pub up_y: f32,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct AreaSize {
    pub width: f32,
    pub height: f32,
}

impl AreaSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

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

// Why is this double mutexed?
type SharedControllerState = Arc<Mutex<GamepadCommand>>;
type SharedTouchEvent = Arc<Mutex<Option<CanvasTouch>>>;

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
                        *controller_state.lock().unwrap() = command;
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

const STATIC_FILES_DIR: Dir = include_dir!("static");

pub struct StateHandle {
    controller_state: SharedControllerState,
    last_canvas_touch_event: SharedTouchEvent,
}

impl StateHandle {
    fn new(
        controller_state: SharedControllerState,
        last_canvas_touch_event: SharedTouchEvent,
    ) -> Self {
        Self {
            controller_state,
            last_canvas_touch_event,
        }
    }

    pub fn get_last_gamepad_command(&self) -> GamepadCommand {
        self.controller_state.lock().unwrap().clone()
    }

    pub fn get_latest_canvas_touch(&self) -> Option<CanvasTouch> {
        self.last_canvas_touch_event.lock().unwrap().clone()
    }
}

pub fn start_remote_controller_server(address: impl Into<std::net::SocketAddr>) -> StateHandle {
    start_remote_controller_server_with_map(address, AreaSize::new(1.0, 1.0))
}

pub fn start_remote_controller_server_with_map(
    address: impl Into<std::net::SocketAddr>,
    map_size: AreaSize,
) -> StateHandle {
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

    let map_size_endpoint = warp::path("map").map(move || warp::reply::json(&map_size));

    let touch_event = SharedTouchEvent::default();
    let touch_event_clone = Arc::clone(&touch_event);
    let shared_touch_event_state = warp::any().map(move || touch_event_clone.clone());

    let canvas_touch_endpoint = warp::path("canvas_touch")
        .and(warp::filters::body::json())
        .and(shared_touch_event_state)
        .map(
            |data: CanvasTouch, touch_event: Arc<Mutex<Option<CanvasTouch>>>| {
                *touch_event.lock().unwrap() = Some(data);
                warp::reply()
            },
        );

    // manually construct paths since this allows us to embed the files into the binary
    let index = warp::path::end().map(|| warp::reply::html(include_str!("../static/index.html")));
    let navigation = warp::path("navigation")
        .map(|| warp::reply::html(include_str!("../static/navigation.html")));

    // This is some weird logic...
    // but it works for now and not like this app will ever be used anywhere important
    // famous last words
    let static_file = warp::path("static").and(warp::path::param()).map(
        |param: String| -> Box<dyn warp::reply::Reply> {
            if let Some(file) = STATIC_FILES_DIR.get_file(&param) {
                if let Some(file_text) = file.contents_utf8() {
                    // yep. Manually checking file extensions
                    // I promise most of my code is less bad
                    // I just don't web
                    // Okay maybe that's a lie
                    if param.ends_with(".html") || param.ends_with(".js") {
                        Box::new(warp::reply::html(file_text))
                    } else if param.ends_with(".css") {
                        Box::new(warp::reply::with_header(
                            file_text,
                            "Content-Type",
                            "text/css",
                        ))
                    } else {
                        Box::new("not found")
                    }
                } else {
                    Box::new("not found")
                }
            } else {
                Box::new("not found")
            }
        },
    );

    let routes = index
        .or(navigation)
        .or(ws)
        .or(canvas_touch_endpoint)
        .or(map_size_endpoint)
        .or(static_file);

    tokio::task::spawn(async move {
        warp::serve(routes).run(address).await;
    });

    StateHandle::new(controller_state, touch_event)
}

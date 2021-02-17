use actix::{clock::Instant, prelude::*};
use actix_files as fs;
use actix_web::{middleware, rt::System, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use log::*;
use serde::Deserialize;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GamepadCommand {
    #[serde(rename = "lx")]
    left_x: f32,
    #[serde(rename = "ly")]
    left_y: f32,
    #[serde(rename = "rx")]
    right_x: f32,
    #[serde(rename = "ry")]
    right_y: f32,
}

struct WebsocketActor {
    controller_state: Arc<ControllerState>,
    last_heartbeat: Instant,
}

impl Actor for WebsocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.start_heartbeat(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, context: &mut Self::Context) {
        // process websocket messages
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                context.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                if let Ok(data) = serde_json::from_str::<GamepadCommand>(&text) {
                    self.controller_state.update(data);
                } else {
                    error!("Failed to parse json");
                }
                // ctx.text(format!("Response to: {}", text));
            }
            Ok(ws::Message::Binary(bin)) => context.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                info!("Websocket connection closed");
                context.close(reason);
                context.stop();
            }
            _ => context.stop(),
        }
    }
}

impl WebsocketActor {
    fn new(controller_state: Arc<ControllerState>) -> Self {
        Self {
            controller_state,
            last_heartbeat: Instant::now(),
        }
    }

    fn start_heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                error!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

async fn ws_index(
    request: HttpRequest,
    stream: web::Payload,
    controller_state: web::Data<ControllerState>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WebsocketActor::new(Arc::clone(&*controller_state)),
        &request,
        stream,
    )
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

fn run_remote_controller(address: String, stop_signal: mpsc::Receiver<()>) {
    let mut system = System::new("remote_control_runner");
    let (tx, rx) = mpsc::channel();

    let controller_state_data = web::Data::new(ControllerState::default());
    let handle = thread::spawn(move || {
        let mut sys = System::new("http-server-internal");
        let server = HttpServer::new(move || {
            App::new()
                .wrap(middleware::Logger::default())
                .app_data(controller_state_data.clone())
                .route("/ws/", web::get().to(ws_index))
                .service(fs::Files::new("/", "static/").index_file("index.html"))
        })
        .bind(address)
        .unwrap()
        .workers(2)
        .run();
        let _ = tx.send(server.clone());
        sys.block_on(async { server.await }).unwrap();
        System::current().stop();
    });

    let srv = rx.recv().unwrap();
    system.block_on(async move {
        stop_signal.recv().unwrap();
        srv.stop(true).await;
    });
    System::current().stop();
    if handle.join().is_err() {
        error!("Server thread encountered an error");
    }
}

struct RemoteControllerRunner {
    stop_sender: mpsc::Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}

impl RemoteControllerRunner {
    fn start(address: String) -> Self {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || run_remote_controller(address, rx));
        Self {
            stop_sender: tx,
            join_handle: Some(handle),
        }
    }
}

impl Drop for RemoteControllerRunner {
    fn drop(&mut self) {
        self.stop_sender.send(()).unwrap();
        if let Some(join_handle) = self.join_handle.take() {
            join_handle.join().unwrap();
        } else {
            panic!("Failed to drop RemoteControllerRunner");
        }
    }
}

fn main() {
    std::env::set_var(
        "RUST_LOG",
        "actix_server=info,actix_web=info,remote_controller=trace",
    );
    env_logger::init();
    let a = RemoteControllerRunner::start("127.0.0.1:8080".to_owned());
    info!("Server listening on localhost:8080");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    info!("dropping");
    drop(a);
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}

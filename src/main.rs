use actix::{clock::Instant, prelude::*};
use actix_files as fs;
use actix_web::{middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use log::*;
use std::time::Duration;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct WebsocketActor {
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
                info!("Text message: {}", text);
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
    fn new() -> Self {
        Self {
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

async fn ws_index(request: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    ws::start(WebsocketActor::new(), &request, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var(
        "RUST_LOG",
        "actix_server=info,actix_web=info,remote_controller=trace",
    );
    env_logger::init();

    info!("Server listening on localhost:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .service(web::resource("/ws/").route(web::get().to(ws_index)))
            .service(fs::Files::new("/", "static/").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

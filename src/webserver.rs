use actix::StreamHandler;
use actix_web::{post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use serde::Deserialize;
use tokio::sync::mpsc::{channel, Sender};

// Useful docs, example for app_data:
// https://docs.rs/actix-web/latest/actix_web/web/struct.Data.html
// https://github.com/actix/actix-web/discussions/2805

#[derive(Deserialize, Debug)]
pub enum WebAction {
  Quit,
  Drum,
}

#[derive(Deserialize, Debug)]
pub struct WebMessage {
  pub message: WebAction,
}

#[post("/api/action")]
async fn action(
  message: web::Json<WebMessage>,
  tx: actix_web::web::Data<Sender<WebMessage>>,
) -> impl Responder {
  tx.send(message.into_inner()).await.unwrap();
  HttpResponse::Ok().body("{}")
}

struct MyWs;

impl actix::Actor for MyWs {
  type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
      Ok(ws::Message::Text(text)) => {
        println!("### text {}", text);
      },
      Ok(ws::Message::Binary(bin)) => {
        println!("### binary");
        ctx.binary(bin);
      },
      _ => (),
    }
  }
}

async fn ws_index(
  req: HttpRequest,
  stream: web::Payload,
  tx: actix_web::web::Data<Sender<WebMessage>>,
) -> Result<HttpResponse, actix_web::Error> {
  let resp = ws::start(MyWs {}, &req, stream);
  println!("{:?}", resp);
  resp
}

#[actix_web::main]
pub async fn serve(tx: Sender<WebMessage>) -> std::io::Result<()> {
  HttpServer::new(move || {
    App::new()
      .app_data(actix_web::web::Data::new(tx.clone()))
      .route("/ws/", web::get().to(ws_index))
      .service(action)
      .service(actix_files::Files::new("/", "./public").index_file("index.html"))
  })
  .bind(("127.0.0.1", 8000))?
  .run()
  .await
}

pub fn start<C>(k: C)
where
  C: Fn(&WebMessage) + Send + 'static,
{
  let (tx, mut rx) = channel::<WebMessage>(100);
  std::thread::spawn(move || {
    serve(tx).unwrap();
  });
  std::thread::spawn(move || loop {
    match rx.blocking_recv() {
      None => break,
      Some(msg) => k(&msg),
    }
  });
}

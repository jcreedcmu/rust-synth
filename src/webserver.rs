use crate::midi;
use actix::StreamHandler;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Receiver, Sender};

// Useful docs, example for app_data:
// https://docs.rs/actix-web/latest/actix_web/web/struct.Data.html
// https://github.com/actix/actix-web/discussions/2805

const CHANNEL_CAPACITY: usize = 100;

#[derive(Deserialize, Debug)]
pub enum WebAction {
  Quit,
  Drum,
}

#[derive(Deserialize, Debug)]
pub struct WebMessage {
  pub message: WebAction,
}

#[derive(Serialize, Debug)]
pub enum SynthMessage {
  Ping(midi::Message),
}

pub enum WebOrSubMessage {
  WebMessage(WebMessage),
  SubMessage(Sender<SynthMessage>),
}

struct MyWs {
  tx: Sender<WebOrSubMessage>,
}

impl actix::Actor for MyWs {
  type Context = ws::WebsocketContext<Self>;
}

fn parse(text: &[u8]) -> Result<WebMessage, Box<dyn std::error::Error>> {
  Ok(serde_json::from_str::<WebMessage>(std::str::from_utf8(
    text,
  )?)?)
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
  fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
    match msg {
      Ok(ws::Message::Text(text)) => {
        match parse(text.as_bytes()) {
          Err(err) => {
            println!("Tried to parse {} but {}", text, err);
          },
          Ok(m) => {
            // XXX this fails if buffer is full
            self.tx.try_send(WebOrSubMessage::WebMessage(m)).unwrap();
          },
        }
      },
      Ok(m) => {
        println!("### websocket message received but not handled: {:?}", m);
      },
      _ => (),
    }
  }
}

async fn ws_index(
  req: HttpRequest,
  stream: web::Payload,
  tx: actix_web::web::Data<Sender<WebOrSubMessage>>,
) -> Result<HttpResponse, actix_web::Error> {
  println!("trying to start websocket server");
  let tx = tx.as_ref().clone();
  let resp = ws::start(MyWs { tx }, &req, stream);
  println!("starting websocket server: {:?}", resp);
  resp
}

struct Channels {
  tx: Sender<WebOrSubMessage>,
  rx: Receiver<SynthMessage>,
}

#[actix_web::main]
pub async fn serve(tx: Sender<WebOrSubMessage>) -> std::io::Result<()> {
  HttpServer::new(move || {
    let (txs, mut rxs) = channel::<SynthMessage>(CHANNEL_CAPACITY);
    println!("here1");
    // XXX bad error handling
    tx.try_send(WebOrSubMessage::SubMessage(txs)).unwrap();
    println!("here2");
    let data = actix_web::web::Data::new(Channels {
      tx: tx.clone(),
      rx: rxs,
    });
    println!("here3");
    App::new()
      .app_data(data)
      .route("/ws/", web::get().to(ws_index))
      .service(actix_files::Files::new("/", "./public").index_file("index.html"))
  })
  .bind(("127.0.0.1", 8000))?
  .run()
  .await
}

pub fn start<C>(k: C)
where
  C: Fn(&WebOrSubMessage) + Send + 'static,
{
  let (tx, mut rx) = channel::<WebOrSubMessage>(CHANNEL_CAPACITY);
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

use crate::midi;
use rocket::{get, routes};
use rocket_ws::{Stream, WebSocket};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Receiver, Sender};

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

struct Channels {
  tx: Sender<WebOrSubMessage>,
  rx: Receiver<SynthMessage>,
}

struct WebsocketSession {
  ch: Channels,
}

#[get("/world")]
fn world() -> &'static str {
  "Hello, world!"
}

#[get("/ws")]
async fn echo(ws: WebSocket, state: &rocket::State<Sender<WebOrSubMessage>>) -> Stream!['static] {
  state
    .send(WebOrSubMessage::WebMessage(WebMessage {
      message: WebAction::Drum,
    }))
    .await
    .unwrap();
  ws.stream(|io| io)
}

fn serve(tx: Sender<WebOrSubMessage>) -> Result<(), rocket::Error> {
  ::rocket::async_main(async move {
    let _rocket = rocket::build()
      .mount("/", rocket::fs::FileServer::from("./public"))
      .mount("/", routes![world, echo])
      .manage(tx)
      .launch()
      .await?;
    Ok(())
  })
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

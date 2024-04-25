use crate::midi;
use rocket::{get, routes};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{Receiver, Sender};

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

#[rocket::main]
async fn serve() -> Result<(), rocket::Error> {
  let _rocket = rocket::build().mount("/", routes![world]).launch().await?;

  Ok(())
}

pub fn start<C>(k: C)
where
  C: Fn(&WebOrSubMessage) + Send + 'static,
{
  std::thread::spawn(move || {
    serve().unwrap();
  });
}

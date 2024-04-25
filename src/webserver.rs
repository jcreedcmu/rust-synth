use crate::midi;
use rocket::{get, routes};
use rocket_ws::{stream::DuplexStream, Message as RocketWsMessage, WebSocket};
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

use rocket::futures::{SinkExt, StreamExt};

fn parse(text: &String) -> Result<WebMessage, Box<dyn std::error::Error + Sync + Send>> {
  Ok(serde_json::from_str::<WebMessage>(text.as_str())?)
}

#[get("/ws")]
async fn ws_serve(
  ws: WebSocket,
  state: &rocket::State<Sender<WebOrSubMessage>>,
) -> rocket_ws::Channel<'static> {
  let tx = state.inner().clone();

  ws.channel(move |mut stream: DuplexStream| {
    Box::pin(async move {
      while let Some(message) = stream.next().await {
        match message {
          Err(e) => {
            println!("Getting next websocket message, got error {:?}", e);
          },
          Ok(m) => {
            if let RocketWsMessage::Text(t) = &m {
              match parse(t) {
                Err(e) => {
                  println!("Parsing msg {}, got JSON parse error {:?}", t, e);
                },
                Ok(m) => {
                  tx.send(WebOrSubMessage::WebMessage(m)).await.unwrap();
                },
              }
            }
          },
        }
      }

      Ok(())
    })
  })
}

fn serve(tx: Sender<WebOrSubMessage>) -> Result<(), rocket::Error> {
  ::rocket::async_main(async move {
    let _rocket = rocket::build()
      .mount("/", rocket::fs::FileServer::from("./public"))
      .mount("/", routes![ws_serve])
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

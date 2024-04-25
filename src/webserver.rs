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
  let (txs, mut rxs) = channel::<SynthMessage>(CHANNEL_CAPACITY);

  state.send(WebOrSubMessage::SubMessage(txs)).await.unwrap();

  ws.channel(move |mut stream: DuplexStream| {
    let (mut sink, mut src) = stream.split();
    Box::pin(async move {
      // handle messages from synth to client
      tokio::spawn(async move {
        while let Some(message) = rxs.recv().await {
          let json_str = serde_json::to_string(&message).unwrap();
          sink.send(RocketWsMessage::Text(json_str)).await.unwrap();
        }
      });
      while let Some(message) = src.next().await {
        // handle message from client to synth
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

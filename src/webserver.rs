use crate::midi;
use rocket::futures::{SinkExt, StreamExt};
use rocket::{get, routes};
use rocket_ws::{stream::DuplexStream, Message as RocketWsMessage, WebSocket};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Sender};

const CHANNEL_CAPACITY: usize = 100;

#[derive(Serialize, Deserialize, Debug)]
pub enum WebAction {
  Quit,
  Drum,
}

// Messages sent from the web client to the synth

#[derive(Serialize, Deserialize, Debug)]
pub struct WebMessage {
  pub message: WebAction,
}

// Messages to the synth, either
// - from the web client, or
// - a converse-direction message subscription request, sent once when
//   we're setting up the websocket connection

pub enum WebOrSubMessage {
  WebMessage(WebMessage),
  SubMessage(Sender<SynthMessage>),
}

// Messages sent from the synthe to the web client

#[derive(Serialize, Debug)]
pub enum SynthMessage {
  Ping(midi::Message),
}

#[get("/ws")]
async fn ws_serve(
  ws: WebSocket,
  state: &rocket::State<Sender<WebOrSubMessage>>,
) -> rocket_ws::Channel<'static> {
  let web_tx = state.inner().clone();
  let (synth_tx, mut synth_rx) = channel::<SynthMessage>(CHANNEL_CAPACITY);

  // Send a "subscription request", i.e. ask the synth to send us messages
  // instead of any clients that might have come before us.
  web_tx
    .send(WebOrSubMessage::SubMessage(synth_tx))
    .await
    .unwrap();

  ws.channel(move |mut stream: DuplexStream| {
    let (mut sink, mut src) = stream.split();
    Box::pin(async move {
      tokio::spawn(async move {
        // handle messages from synth to client
        while let Some(message) = synth_rx.recv().await {
          let json_str = serde_json::to_string(&message).unwrap();
          sink.send(RocketWsMessage::Text(json_str)).await.unwrap();
        }
      });

      // handle messages from client to synth
      while let Some(message) = src.next().await {
        match message {
          Err(e) => {
            println!("Getting next websocket message, got error {:?}", e);
          },
          Ok(m) => {
            if let RocketWsMessage::Text(t) = &m {
              match serde_json::from_str::<WebMessage>(t.as_str()) {
                Err(e) => {
                  println!("Parsing msg {}, got JSON parse error {:?}", t, e);
                },
                Ok(m) => {
                  web_tx.send(WebOrSubMessage::WebMessage(m)).await.unwrap();
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
  let (web_tx, mut web_rx) = channel::<WebOrSubMessage>(CHANNEL_CAPACITY);
  std::thread::spawn(move || {
    serve(web_tx).unwrap();
  });
  std::thread::spawn(move || loop {
    match web_rx.blocking_recv() {
      None => break,
      Some(msg) => k(&msg),
    }
  });
}

#[cfg(test)]
mod tests {
  use crate::webserver::{WebAction, WebMessage};
  #[test]
  fn web_message_serialization() {
    let message = WebMessage {
      message: WebAction::Quit,
    };
    let json_str = serde_json::to_string(&message).unwrap();

    assert_eq!(json_str, r###"{"message":"Quit"}"###);
  }
}

use crate::midi;
use crate::state::ControlBlock;
use crate::ugen::UgenSpec;
use crate::util::UnitHandle;
use rocket::futures::{SinkExt, StreamExt};
use rocket::{get, routes};
use rocket_ws::{stream::DuplexStream, Message as RocketWsMessage, WebSocket};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{channel, Sender};
use ts_rs::TS;

const CHANNEL_CAPACITY: usize = 100;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub enum WebMessage {
  Quit,
  Drum,
  SetControlBlock { index: usize, ctl: ControlBlock },
  SetSequencer { inst: usize, pat: usize, on: bool },
  Reconfigure { specs: Vec<UgenSpec> },
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
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub enum SynthMessage {
  Midi {
    msg: midi::Message,
  },
  Meter {
    level: f32, // rms
    peak: f32,
  },
}

#[get("/ws")]
// Note that we could also have said
// impl rocket::response::Responder
// as the return type here
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

fn serve(tx: Sender<WebOrSubMessage>) -> anyhow::Result<()> {
  ::rocket::async_main(async move {
    let _rocket = rocket::build()
      .configure(rocket::Config {
        shutdown: rocket::config::Shutdown {
          ctrlc: false,
          ..Default::default()
        },
        ..Default::default()
      })
      .mount("/", rocket::fs::FileServer::from("./public"))
      .mount("/", routes![ws_serve])
      .manage(tx)
      .launch()
      .await?;
    Ok(())
  })
}

pub fn start<C>(k: C) -> (UnitHandle, UnitHandle)
where
  C: Fn(WebOrSubMessage) -> anyhow::Result<()> + Send + 'static,
{
  let (web_tx, mut web_rx) = channel::<WebOrSubMessage>(CHANNEL_CAPACITY);
  let serve_thread = std::thread::spawn(move || {
    serve(web_tx).unwrap();
  });
  let fwd_thread = std::thread::spawn(move || loop {
    match web_rx.blocking_recv() {
      None => {
        println!("web_rx blocking recv got None, in fwd_thread");
        break;
      },
      Some(msg) => k(msg).unwrap(),
    }
  });
  (serve_thread, fwd_thread)
}

#[cfg(test)]
mod tests {
  use crate::webserver::WebMessage;
  #[test]
  fn quit_message_serialization() {
    let message = WebMessage::Quit;
    let json_str = serde_json::to_string(&message).unwrap();
    assert_eq!(json_str, r###"{"t":"quit"}"###);
  }

  #[test]
  fn set_volume_message_serialization() {
    let message = WebMessage::SetSequencer {
      inst: 123,
      on: true,
      pat: 234,
    };
    let json_str = serde_json::to_string(&message).unwrap();
    assert_eq!(
      json_str,
      r###"{"t":"setSequencer","inst":123,"pat":234,"on":true}"###
    );
  }
}

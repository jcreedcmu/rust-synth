use crate::midi;
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

pub fn start<C>(k: C)
where
  C: Fn(&WebOrSubMessage) + Send + 'static,
{
}

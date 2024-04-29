use std::error::Error;
use std::fmt;

use anyhow::{anyhow, bail};
use midir::{Ignore, MidiInput};
use serde::Serialize;

pub struct MidiService {
  conn_in: midir::MidiInputConnection<()>,
}

type Pitch = u8;

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
pub enum Message {
  NoteOn {
    pitch: Pitch,
    channel: u8,
    velocity: u8,
  },
  NoteOff {
    pitch: Pitch,
    channel: u8,
  },
  PedalOn,
  PedalOff,
}

use self::Message::*;

fn message_of_vec(vec: &[u8]) -> Option<Message> {
  match vec.len() {
    3 => match vec[0] {
      0x80..=0x8f => Some(NoteOff {
        channel: vec[0] - 0x80,
        pitch: vec[1],
      }),
      0x90..=0x9f => {
        if vec[2] != 0 {
          Some(NoteOn {
            channel: vec[0] - 0x90,
            pitch: vec[1],
            velocity: vec[2],
          })
        } else {
          Some(NoteOff {
            channel: 0,
            pitch: vec[1],
          })
        }
      },
      0xb0 => match vec[1] {
        0x40 => match vec[2] {
          0x00 => Some(PedalOff),
          _ => Some(PedalOn),
        },
        _ => None,
      },
      _ => None,
    },
    _ => None,
  }
}

impl MidiService {
  pub fn new<C>(source_index: usize, k: C) -> anyhow::Result<MidiService>
  where
    C: Fn(&Message) -> anyhow::Result<()> + std::marker::Send + Sync + 'static,
  {
    let mut midi_in = MidiInput::new("midir input")?;
    midi_in.ignore(Ignore::None);

    let midi_device_num = 1;
    let in_port = midi_in
      .ports()
      .get(midi_device_num)
      .ok_or(anyhow!("Invalid port number"))?
      .clone();

    println!("\nOpening connections");
    let in_port_name = midi_in.port_name(&in_port)?;

    let conn_in_result = midi_in.connect(
      &in_port,
      "midir-print",
      move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
        match message_of_vec(message) {
          Some(msg) => match k(&msg) {
            Ok(()) => (),
            Err(e) => println!("Error in midi callback: {}", e.to_string()),
          },
          None => println!("Warning, unknown midi message: {:?}", message),
        }
      },
      (),
    );

    let conn_in: midir::MidiInputConnection<()> = match conn_in_result {
      Ok(v) => v,
      Err(e) => bail!("Error: Can't make midi input connection: {}", e.to_string()),
    };

    Ok(MidiService { conn_in })
  }
}

#[derive(Debug)]
pub enum MidiError {
  Os(i32),
  BadSource(usize),
}

impl From<i32> for MidiError {
  fn from(x: i32) -> MidiError {
    MidiError::Os(x)
  }
}

impl fmt::Display for MidiError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MidiError::Os(x) => write!(f, "Os error {}", x),
      MidiError::BadSource(x) => write!(f, "Bad source {}", x),
    }
  }
}

impl Error for MidiError {
  fn description(&self) -> &str {
    "invalid first item to double"
  }

  fn cause(&self) -> Option<&dyn Error> {
    // Generic error, underlying cause isn't tracked.
    None
  }
}

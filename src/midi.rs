use std::error::Error;
use std::fmt;

use midir::{Ignore, MidiInput};

pub struct MidiService {
  conn_in: midir::MidiInputConnection<()>,
}

type Pitch = u8;

#[derive(Debug)]
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
      }
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
  pub fn new<C>(source_index: usize, k: C) -> Result<MidiService, Box<dyn Error>>
  where
    C: Fn(&Message) + std::marker::Send + 'static,
  {
    let mut midi_in = MidiInput::new("midir input")?;
    midi_in.ignore(Ignore::None);

    let midi_device_num = 1;
    let in_port = midi_in
      .ports()
      .get(midi_device_num)
      .ok_or("Invalid port number")?
      .clone();

    println!("\nOpening connections");
    let in_port_name = midi_in.port_name(&in_port)?;

    // _conn_in needs to be a named binding, because it needs to be kept alive until the end of the scope
    let conn_in: midir::MidiInputConnection<()> = midi_in.connect(
      &in_port,
      "midir-print",
      move |stamp, message, _| {
        println!("{}: {:?} (len = {})", stamp, message, message.len());
        if let Some(msg) = message_of_vec(message) {
          k(&msg);
        }
      },
      (),
    )?;

    Ok(MidiService { conn_in })
  }
}

// pub fn input_port<F>(&self, name: &str, callback: F) -> Result<InputPort, OSStatus>
//         where F: FnMut(&PacketList) + Send + 'static {

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

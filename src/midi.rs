extern crate coremidi;
use coremidi::{Client, PacketList, Source, Sources};
use std::env;
use std::error::Error;
use std::fmt;
use std::thread::sleep;
use std::time::Duration;

pub struct MidiService {
  client: Client,
  input_port: coremidi::InputPort,
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
      0x80...0x8f => Some(NoteOff {
        channel: vec[0] - 0x80,
        pitch: vec[1],
      }),
      0x90...0x9f => {
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
  pub fn new<C>(source_index: usize, k: C) -> Result<MidiService, MidiError>
  where
    C: Fn(&Message) + std::marker::Send + 'static,
  {
    let source = match Source::from_index(source_index) {
      Some(x) => x,
      None => Err(MidiError::BadSource(source_index))?,
    };

    let client = Client::new("example-client")?;
    println!("Listening...");
    let input_port = client.input_port("example-port", move |packet_list: &PacketList| {
      for x in packet_list.iter() {
        println!("{}", x);
        if let Some(x) = message_of_vec(&x.data()) {
          k(&x);
        }
      }
    })?;
    input_port.connect_source(&source)?;

    //  input_port.disconnect_source(&source)?;
    Ok(MidiService { client, input_port })
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

  fn cause(&self) -> Option<&Error> {
    // Generic error, underlying cause isn't tracked.
    None
  }
}

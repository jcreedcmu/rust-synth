#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
extern crate midir;

mod audio;
mod beep;
mod util;

use midir::{Ignore, MidiIO, MidiInput, MidiInputPort, MidiOutput};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex, MutexGuard};

const BOTTOM_NOTE: u8 = 21;
const NUM_NOTES: usize = 88;

#[derive(Clone, Debug)]
pub enum NoteFsm {
  On { amp: f64, t: f64, vel: f64 },
  Release { amp: f64, t: f64 },
}

#[derive(Clone, Debug)]
pub struct NoteState {
  pitch: u8,
  freq: f64,
  phase: f64,
  fsm: NoteFsm,
}

#[derive(Clone, Debug)]
pub struct KeyState {
  is_on: Option<usize>, // index into note_state vector
}

#[derive(Debug)]
pub struct State {
  phase: f64,
  freq: f64,
  key_state: Vec<KeyState>,
  note_state: Vec<Option<NoteState>>,
  write_to_file: bool,
}

pub struct Data {
  state: Arc<Mutex<State>>,
}

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn do_midi_stuff() -> Result<(), Box<dyn Error>> {
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
  let _conn_in = midi_in.connect(
    &in_port,
    "midir-print",
    move |stamp, message, _| {
      println!("{}: {:?} (len = {})", stamp, message, message.len());
    },
    (),
  )?;

  let mut input = String::new();
  stdin().read_line(&mut input)?; // wait for next enter key press
  Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
  let state = Arc::new(Mutex::new(State {
    phase: 0.0,
    freq: 440.0,
    key_state: vec![KeyState { is_on: None }; NUM_NOTES],
    note_state: vec![],
    write_to_file: false,
  }));

  let _ = std::thread::spawn(move || {
    let _ = do_midi_stuff();
  });

  let ads = audio::AudioService::new(&Data { state })?;
  Ok(())
}

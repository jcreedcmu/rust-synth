#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
extern crate midir;

mod audio;
mod beep;
mod midi;
mod util;

use midir::{Ignore, MidiIO, MidiInput, MidiInputPort, MidiOutput};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex, MutexGuard};

const BOTTOM_NOTE: u8 = 21;
const NUM_NOTES: usize = 88;

#[derive(Clone, Debug)]
pub enum NoteFsm {
  On { amp: f32, t: f32, vel: f32 },
  Release { amp: f32, t: f32 },
}

#[derive(Clone, Debug)]
pub struct NoteState {
  pitch: u8,
  freq: f32,
  phase: f32,
  fsm: NoteFsm,
}

#[derive(Clone, Debug)]
pub struct KeyState {
  is_on: Option<usize>, // index into note_state vector
}

#[derive(Debug)]
pub struct State {
  phase: f32,
  freq: f32,
  going: bool,
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

fn run() -> Result<(), Box<dyn Error>> {
  let state = Arc::new(Mutex::new(State {
    phase: 0.0,
    freq: 440.0,
    going: true,
    key_state: vec![KeyState { is_on: None }; NUM_NOTES],
    note_state: vec![],
    write_to_file: false,
  }));

  let sg = Data {
    state: state.clone(),
  };

  let do_midi_stuff = move || -> Result<(), Box<dyn Error>> {
    let ms = midi::MidiService::new(0, move |msg: &midi::Message| {
      println!("midi message processed!");
    });
    let mut input = String::new();
    stdin().read_line(&mut input)?; // wait for next enter key press

    let mut s: MutexGuard<State> = sg.state.lock().unwrap();
    s.going = false;
    Ok(())
  };

  let _ = std::thread::spawn(move || {
    let _ = do_midi_stuff();
  });

  let ads = audio::AudioService::new(&Data { state })?;
  Ok(())
}

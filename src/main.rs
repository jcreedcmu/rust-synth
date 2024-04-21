#![allow(unused_variables, unused_mut, dead_code)]
extern crate midir;

mod audio;
mod consts;
mod midi;
mod reduce;
mod synth;
mod util;

use consts::{Data, KeyState, State, NUM_NOTES};
use midi::Message;
use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex, MutexGuard};

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

  let sg2 = Data {
    state: state.clone(),
  };

  let _ = std::thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
    // XXX MidiService should just mut-borrow state?
    let ms = midi::MidiService::new(0, move |msg: &Message| {
      let mut s: MutexGuard<State> = sg.state.lock().unwrap();
      reduce::midi_reducer(msg, &mut s);
    });
    let mut input = String::new();
    stdin().read_line(&mut input)?; // wait for next enter key press

    let mut s: MutexGuard<State> = sg2.state.lock().unwrap();
    s.going = false;
    Ok(())
  });

  let ads = audio::AudioService::new(&Data { state }, synth::Synth::new())?;
  Ok(())
}

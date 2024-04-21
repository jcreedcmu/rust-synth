#![allow(unused_variables, unused_mut, dead_code, non_upper_case_globals)]
extern crate midir;

mod audio;
mod consts;
mod midi;
mod reduce;
mod state;
mod synth;
mod util;

use midi::Message;
use state::{Data, State};
use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::consts::AUDIO_CARD;

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let state = Arc::new(Mutex::new(State::new()));

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

  let ads = audio::AudioService::new(AUDIO_CARD, &Data { state }, synth::Synth::new())?;
  Ok(())
}

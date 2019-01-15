#![feature(core_intrinsics, try_trait)]
#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
#![feature(uniform_paths)]

//! Play some sounds.

mod audio;
mod midi;
mod sb;
mod util;

use midi::Message;
use std::error::Error;
use std::option::NoneError;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{sleep, spawn};
use std::time::Duration;
use util::Mostly;

fn main() {
  match run() {
    Ok(_) => {}
    e => {
      eprintln!("Example failed with the following: {:?}", e);
    }
  }
}

pub struct State {
  phase: f64,
  freq: f64,
}

pub struct Data {
  state: Arc<Mutex<State>>,
}

fn run() -> Mostly<()> {
  let state = Arc::new(Mutex::new(State {
    phase: 0.0,
    freq: 440.0,
  }));

  let dcb = Data {
    state: state.clone(),
  };

  let ms = midi::MidiService::new(0, move |msg: &Message| {
    let mut s: MutexGuard<State> = dcb.state.lock().unwrap();
    match msg {
      Message::NoteOn {
        pitch,
        channel,
        velocity,
      } => {
        s.freq = 440.0 * 2.0f64.powf(((*pitch as f64) - 69.0) / 12.0);
      }
      Message::NoteOff { pitch, channel } => {
        s.freq = 0.0;
      }
      _ => (),
    }
    println!("{:?}", msg);
  })?;

  let ads = audio::AudioService::new(&Data { state })?;

  sleep(Duration::from_millis(25000));

  Ok(())
}

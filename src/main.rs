#![feature(core_intrinsics, try_trait)]
#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
#![feature(uniform_paths)]

//! Play some sounds.

mod audio;
mod midi;
mod sb;
mod util;

use std::error::Error;
use std::option::NoneError;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
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

pub struct Data {
  phase: Arc<Mutex<f64>>,
  freq: Arc<Mutex<f64>>,
}

fn run() -> Mostly<()> {
  let state = Data {
    phase: Arc::new(Mutex::new(0.0)),
    freq: Arc::new(Mutex::new(440.0)),
  };

  //  sb::dance();
  let ads = audio::AudioService::new(&state)?;
  // let ms = midi::MidiService::new(0, move |msg| {
  //   println!("{:?}", msg);
  // })?;

  sleep(Duration::from_millis(25000));

  Ok(())
}

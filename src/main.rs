#![feature(core_intrinsics, try_trait)]
#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
#![feature(uniform_paths)]

//! Play some sounds.

mod audio;
mod midi;
mod util;

use std::error::Error;
use std::option::NoneError;
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

fn run() -> Mostly<()> {
  // do_other()?;
  let ms = midi::MidiService::new(0, move |msg| {
    println!("{:?}", msg);
  })?;

  sleep(Duration::from_millis(25000));

  Ok(())
}

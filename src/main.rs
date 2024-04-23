#![allow(unused_variables, unused_mut, dead_code, non_upper_case_globals)]
extern crate midir;

mod audio;
mod bass_drum;
mod consts;
mod midi;
mod reasonable_synth;
mod reduce;
mod state;
mod synth;
mod ugen;
mod ugen_factory;
mod util;

use midi::{Message, MidiService};
use reduce::add_ugen_state;
use state::{Data, State};
use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex, MutexGuard};
use ugen_factory::UgenFactory;

use crate::consts::AUDIO_CARD;

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn mk_sequencer_thread(sg: Arc<Mutex<State>>, fac: UgenFactory) {
  std::thread::spawn(move || {
    let mut toggle: bool = true;
    loop {
      std::thread::sleep(std::time::Duration::from_millis(500));
      {
        let mut s: MutexGuard<State> = sg.lock().unwrap();
        if !s.going {
          break;
        }
        add_ugen_state(&mut s, fac.new_drum(if toggle { 660.0 } else { 1760.0 }));
        toggle = !toggle;
      }
    }
  });
}

fn mk_midi_service(sg: Arc<Mutex<State>>, fac: UgenFactory) -> Result<MidiService, Box<dyn Error>> {
  // XXX MidiService should just mut-borrow state?
  midi::MidiService::new(0, move |msg: &Message| {
    let mut s: MutexGuard<State> = sg.lock().unwrap();
    reduce::midi_reducer(msg, &fac, &mut s);
  })
}

fn mk_stdin_thread(sg: Arc<Mutex<State>>, fac: UgenFactory) {
  std::thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
      let mut input = String::new();
      stdin().read_line(&mut input)?; // wait for next enter key press

      match input.as_str() {
        "\n" => {
          let mut s: MutexGuard<State> = sg.lock().unwrap();
          s.going = false;
          break;
        },
        "k\n" => {
          let mut s: MutexGuard<State> = sg.lock().unwrap();
          add_ugen_state(&mut s, fac.new_drum(440.0));
        },
        _ => println!("Didn't recognize {input}."),
      }
    }
    Ok(())
  });
}

fn run() -> Result<(), Box<dyn Error>> {
  let state = Arc::new(Mutex::new(State::new()));
  let fac = UgenFactory::new();

  let ms = mk_midi_service(state.clone(), fac.clone())?;
  mk_sequencer_thread(state.clone(), fac.clone());
  mk_stdin_thread(state.clone(), fac.clone());

  let ads = audio::AudioService::new(AUDIO_CARD, &Data { state }, synth::Synth::new())?;
  Ok(())
}

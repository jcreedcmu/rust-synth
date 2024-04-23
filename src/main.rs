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
mod util;
mod wavetables;

use midi::{Message, MidiService};
use reduce::add_ugen_state;
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

fn mk_sequencer_thread(sg: Arc<Mutex<State>>) {
  std::thread::spawn(move || {
    let mut toggle: bool = true;
    loop {
      std::thread::sleep(std::time::Duration::from_millis(500));
      {
        let mut s: MutexGuard<State> = sg.lock().unwrap();
        if !s.going {
          break;
        }
        let ugen = s.new_drum(if toggle { 660.0 } else { 1760.0 });
        add_ugen_state(&mut s, ugen);
        toggle = !toggle;
      }
    }
  });
}

fn mk_midi_service(sg: Arc<Mutex<State>>) -> Result<MidiService, Box<dyn Error>> {
  midi::MidiService::new(0, move |msg: &Message| {
    let mut s: MutexGuard<State> = sg.lock().unwrap();
    reduce::midi_reducer(msg, &mut s);
  })
}

fn mk_stdin_thread(sg: Arc<Mutex<State>>) {
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
          let ugen = s.new_drum(440.0);
          add_ugen_state(&mut s, ugen);
        },
        _ => println!("Didn't recognize {input}."),
      }
    }
    Ok(())
  });
}

fn run() -> Result<(), Box<dyn Error>> {
  let state = Arc::new(Mutex::new(State::new()));

  let ms = mk_midi_service(state.clone())?;
  mk_sequencer_thread(state.clone());
  mk_stdin_thread(state.clone());

  let ads = audio::AudioService::new(AUDIO_CARD, &Data { state }, synth::Synth::new())?;
  Ok(())
}

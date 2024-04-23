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

use bass_drum::BassDrumSynthState;
use midi::Message;
use reduce::add_ugen_state;
use state::{Data, State, UgenState};
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

  let sg3 = Data {
    state: state.clone(),
  };

  let _ = std::thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut toggle: bool = true;
    loop {
      std::thread::sleep(std::time::Duration::from_millis(500));
      {
        let mut s: MutexGuard<State> = sg3.state.lock().unwrap();
        if !s.going {
          break;
        }
        add_ugen_state(
          &mut s,
          UgenState::BassDrumSynth(BassDrumSynthState::new(if toggle { 660.0 } else { 1760.0 })),
        );
        toggle = !toggle;
      }
    }
    Ok(())
  });

  let _ = std::thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
    // XXX MidiService should just mut-borrow state?
    let ms = midi::MidiService::new(0, move |msg: &Message| {
      let mut s: MutexGuard<State> = sg.state.lock().unwrap();
      reduce::midi_reducer(msg, &mut s);
    });

    loop {
      let mut input = String::new();
      stdin().read_line(&mut input)?; // wait for next enter key press

      match input.as_str() {
        "\n" => {
          let mut s: MutexGuard<State> = sg2.state.lock().unwrap();
          s.going = false;
          break;
        },
        "k\n" => {
          let mut s: MutexGuard<State> = sg2.state.lock().unwrap();
          add_ugen_state(
            &mut s,
            UgenState::BassDrumSynth(BassDrumSynthState::new(440.0)),
          );
        },
        _ => println!("Didn't recognize {input}."),
      }
    }
    Ok(())
  });

  let ads = audio::AudioService::new(AUDIO_CARD, &Data { state }, synth::Synth::new())?;
  Ok(())
}

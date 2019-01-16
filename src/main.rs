#![feature(core_intrinsics, try_trait)]
#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
#![feature(uniform_paths)]

//! Play some sounds.

mod audio;
mod midi;
mod sb;
mod util;

use audio::note_fsm_amp;
use midi::Message;
use std::error::Error;
use std::option::NoneError;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{sleep, spawn};
use std::time::Duration;
use util::Mostly;

const BOTTOM_NOTE: u8 = 21;
const NUM_NOTES: usize = 88;

fn main() {
  match run() {
    Ok(_) => {}
    e => {
      eprintln!("Example failed with the following: {:?}", e);
    }
  }
}

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
}

pub struct Data {
  state: Arc<Mutex<State>>,
}

fn find_note(s: &State, pitch: u8) -> Option<usize> {
  s.note_state.iter().position(|x| match x {
    Some(y) => y.pitch == pitch,
    _ => false,
  })
}

fn add_note(ns: &mut Vec<Option<NoteState>>, new: NoteState) -> () {
  match ns.iter().position(|x| match x {
    None => true,
    _ => false,
  }) {
    None => ns.push(Some(new)),
    Some(i) => ns[i] = Some(new),
  }
}

fn restrike_note(note: &mut NoteState, vel: f64) {
  note.fsm = NoteFsm::On {
    t: 0.0,
    amp: note_fsm_amp(&note.fsm),
    vel,
  };
}

fn release_note(note: &mut Option<NoteState>) {
  match note {
    Some(NoteState { ref mut fsm, .. }) => {
      *fsm = NoteFsm::Release {
        t: 0.0,
        amp: note_fsm_amp(fsm),
      };
    }
    _ => (),
  }
}

fn run() -> Mostly<()> {
  let state = Arc::new(Mutex::new(State {
    phase: 0.0,
    freq: 440.0,
    key_state: vec![KeyState { is_on: None }; NUM_NOTES],
    note_state: vec![],
  }));

  let dcb = Data {
    state: state.clone(),
  };

  let ms = midi::MidiService::new(0, move |msg: &Message| {
    match msg {
      Message::NoteOn {
        pitch,
        channel,
        velocity,
      } => {
        let pitch = *pitch;
        let freq = 440.0 * 2.0f64.powf(((pitch as f64) - 69.0) / 12.0);
        let mut s: MutexGuard<State> = dcb.state.lock().unwrap();
        // Is this note already being played?
        let pre = find_note(&s, pitch);
        let vel = (*velocity as f64) / 1280.0;
        match pre {
          Some(i) => match &mut s.note_state[i] {
            None => panic!("we thought this note already existed"),
            Some(ref mut ns) => restrike_note(ns, vel),
          },
          None => add_note(
            &mut s.note_state,
            NoteState {
              phase: 0.0,
              freq,
              pitch,
              fsm: NoteFsm::On {
                amp: 0.0,
                t: 0.0,
                vel,
              },
            },
          ),
        }
      }
      Message::NoteOff { pitch, channel } => {
        let mut s: MutexGuard<State> = dcb.state.lock().unwrap();
        let pre = find_note(&s, *pitch);

        match pre {
          None => println!("kinda weird, a noteoff {} on something already off", pitch),
          Some(i) => {
            release_note(&mut (s.note_state[i]));
          }
        }
      }
      _ => (),
    }
    println!("{:?}", msg);
  })?;

  let ads = audio::AudioService::new(&Data { state })?;

  Ok(())
}

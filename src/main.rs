#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
extern crate midir;

mod audio;
mod consts;
mod midi;
mod synth;
mod util;

use consts::{Data, KeyState, NoteFsm, NoteState, State, NUM_NOTES};
use midi::Message;
use midir::{Ignore, MidiIO, MidiInput, MidiInputPort, MidiOutput};
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex, MutexGuard};
use synth::note_fsm_amp;
use util::Mostly;

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
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

fn restrike_note(note: &mut NoteState, vel: f32) {
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

fn midi_reducer(msg: &Message, s: &mut State) {
  match msg {
    Message::NoteOn {
      pitch,
      channel,
      velocity,
    } => {
      let pitch = *pitch;
      let freq = 440.0 * 2.0f32.powf(((pitch as f32) - 69.0) / 12.0);
      // Is this note already being played?
      let pre = find_note(&s, pitch);
      let vel = (*velocity as f32) / 1280.0;
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
      midi_reducer(msg, &mut s);
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

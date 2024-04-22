use std::slice::IterMut;
use std::sync::{Arc, Mutex};

use crate::consts::{BOTTOM_NOTE, NUM_KEYS};

// This is the part of the state that tracks where a note is in its
// ADSR envelope.
#[derive(Clone, Debug)]
pub enum EnvState {
  // Note is activeply sounding. Its pre-existing amplitude at onset
  // time is `amp`. The goal amplitude, at the peak of attack, is
  // `vel`. The amount of time elapsed since its onset is `t_s`.
  On { amp: f32, t_s: f32, vel: f32 },
  // Note is no longer activeply sounding. Its pre-existing amplitude
  // at time of release is `amp`. The amount of time elapsed since its
  // release is `t_s`.
  Release { amp: f32, t_s: f32 },
}

#[derive(Clone, Debug)]
pub struct BassDrumSynthState {
  pub t_s: f32,
  pub freq_hz: f32,
  pub phase: f32,
}

#[derive(Clone, Debug)]
pub struct ReasonableSynthState {
  pub pitch: u8,
  pub freq_hz: f32,
  pub phase: f32,
  pub env_state: EnvState,
}

#[derive(Clone, Debug)]
pub enum UgenState {
  ReasonableSynth(ReasonableSynthState),
  BassDrumSynth(BassDrumSynthState),
}

#[derive(Clone, Debug)]
pub enum KeyState {
  Off,
  On { ugen_ix: usize },   // index into ugen_state vector
  Held { ugen_ix: usize }, // only on because pedal held
}

#[derive(Debug)]
pub struct State {
  pub going: bool,

  // This is NUM_KEYS long, one keystate for every physical key.
  key_state: Vec<KeyState>,

  // This has a varying length as synthesis goes on. Every time we
  // need to allocate a ugen, we try to reuse existing `None`s, but
  // push a new one if necessary.
  pub ugen_state: Vec<Option<UgenState>>,

  // Is the sustain pedal on?
  pub pedal: bool,

  pub write_to_file: bool,
}

pub struct Data {
  pub state: Arc<Mutex<State>>,
}

impl State {
  pub fn new() -> State {
    State {
      going: true,
      key_state: vec![KeyState::Off; NUM_KEYS],
      ugen_state: vec![],
      pedal: false,
      write_to_file: true,
    }
  }

  pub fn get_key_state_mut(self: &mut State, pitch: usize) -> &mut KeyState {
    &mut self.key_state[pitch - (BOTTOM_NOTE as usize)]
  }

  pub fn get_key_states(self: &mut State) -> IterMut<'_, KeyState> {
    self.key_state.iter_mut()
  }
}

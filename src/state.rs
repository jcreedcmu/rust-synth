use std::slice::IterMut;
use std::sync::{Arc, Mutex};

use crate::bass_drum::BassDrumSynthState;
use crate::consts::{BOTTOM_NOTE, NUM_KEYS};
use crate::reasonable_synth::ReasonableSynthState;
use crate::ugen::Ugen;
use crate::wavetables::Wavetables;

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
  pub ugen_state: Vec<Option<Box<dyn Ugen>>>,

  // Is the sustain pedal on?
  pub pedal: bool,

  pub write_to_file: bool,

  pub wavetables: Wavetables,
}

// XXX I don't need this wrapper, do I?
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
      wavetables: Wavetables::new(),
    }
  }

  pub fn get_key_state_mut(self: &mut State, pitch: usize) -> &mut KeyState {
    &mut self.key_state[pitch - (BOTTOM_NOTE as usize)]
  }

  pub fn get_key_states(self: &mut State) -> IterMut<'_, KeyState> {
    self.key_state.iter_mut()
  }

  pub fn new_reasonable(&self, freq_hz: f32, vel: f32) -> ReasonableSynthState {
    ReasonableSynthState::new(freq_hz, vel, self.wavetables.saw_wavetable.clone())
  }

  pub fn new_drum(&self, freq_hz: f32) -> BassDrumSynthState {
    BassDrumSynthState::new(freq_hz, self.wavetables.noise_wavetable.clone())
  }
}

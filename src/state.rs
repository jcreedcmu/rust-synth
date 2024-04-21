use std::sync::{Arc, Mutex};

use crate::consts::NUM_NOTES;

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
pub struct NoteState {
  pub pitch: u8,
  pub freq_hz: f32,
  pub phase: f32,
  pub env_state: EnvState,
}

#[derive(Clone, Debug)]
pub struct KeyState {
  pub is_on: Option<usize>, // index into note_state vector
}

#[derive(Debug)]
pub struct State {
  pub going: bool,
  pub key_state: Vec<KeyState>,
  pub note_state: Vec<Option<NoteState>>,
  pub write_to_file: bool,
}

pub struct Data {
  pub state: Arc<Mutex<State>>,
}

impl State {
  pub fn new() -> State {
    State {
      going: true,
      key_state: vec![KeyState { is_on: None }; NUM_NOTES],
      note_state: vec![],
      write_to_file: false,
    }
  }
}

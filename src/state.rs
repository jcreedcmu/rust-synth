use std::sync::{Arc, Mutex};

use crate::consts::NUM_NOTES;

#[derive(Clone, Debug)]
pub enum NoteFsm {
  On { amp: f32, t_s: f32, vel: f32 },
  Release { amp: f32, t_s: f32 },
}

#[derive(Clone, Debug)]
pub struct NoteState {
  pub pitch: u8,
  pub freq_hz: f32,
  pub phase: f32,
  pub fsm: NoteFsm,
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

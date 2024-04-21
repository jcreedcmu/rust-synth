use std::sync::{Arc, Mutex, MutexGuard};

pub const BOTTOM_NOTE: u8 = 21;
pub const NUM_NOTES: usize = 88;

pub const SAMPLE_RATE: f32 = 44_100.0;

#[derive(Clone, Debug)]
pub enum NoteFsm {
  On { amp: f32, t: f32, vel: f32 },
  Release { amp: f32, t: f32 },
}

#[derive(Clone, Debug)]
pub struct NoteState {
  pub pitch: u8,
  pub freq: f32,
  pub phase: f32,
  pub fsm: NoteFsm,
}

#[derive(Clone, Debug)]
pub struct KeyState {
  pub is_on: Option<usize>, // index into note_state vector
}

#[derive(Debug)]
pub struct State {
  pub phase: f32,
  pub freq: f32,
  pub going: bool,
  pub key_state: Vec<KeyState>,
  pub note_state: Vec<Option<NoteState>>,
  pub write_to_file: bool,
}

pub struct Data {
  pub state: Arc<Mutex<State>>,
}

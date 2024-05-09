use crate::consts::NUM_KEYS;
use crate::state::{AudioBusses, ControlBlocks, KeyState};
use crate::ugen::Ugen;

#[derive(Clone, Debug)]
pub struct MidiManagerState {
  dst: usize,
  // Is the sustain pedal on?
  pub pedal: bool,
  // This is NUM_KEYS long, one keystate for every physical key.
  pub key_state: Vec<KeyState>,
}

impl MidiManagerState {
  pub fn new(dst: usize) -> MidiManagerState {
    MidiManagerState {
      dst,
      pedal: false,
      key_state: vec![KeyState::Off; NUM_KEYS],
    }
  }
}

impl Ugen for MidiManagerState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    true
  }
  fn release(&mut self) {}
  fn restrike(&mut self, _vel: f32) {}
}

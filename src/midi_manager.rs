use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen::Ugen;

#[derive(Clone, Debug)]
pub struct MidiManagerState {
  dst: usize,
  // Is the sustain pedal on?
  pub pedal: bool,
}

impl MidiManagerState {
  pub fn new(dst: usize) -> MidiManagerState {
    MidiManagerState { dst, pedal: false }
  }
}

impl Ugen for MidiManagerState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    true
  }
  fn release(&mut self) {}
  fn restrike(&mut self, _vel: f32) {}
}

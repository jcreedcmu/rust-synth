use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen::Ugen;

#[derive(Clone, Debug)]
pub struct MidiManagerState {
  dst: usize,
}

impl MidiManagerState {
  pub fn new(dst: usize) -> MidiManagerState {
    MidiManagerState { dst }
  }
}

impl Ugen for MidiManagerState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    true
  }
  fn release(&mut self) {}
  fn restrike(&mut self, _vel: f32) {}
}

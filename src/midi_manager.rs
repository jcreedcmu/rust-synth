use std::sync::Arc;

use crate::consts::BUS_DRY;
use crate::envelope::Adsr;
use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen::Ugen;

#[derive(Clone, Debug)]
pub struct MidiManagerState {
  dst: usize,
}

impl MidiManagerState {
  pub fn new(
    freq_hz: f32,
    freq2_hz: f32,
    adsr: Adsr,
    wavetable: Arc<Vec<f32>>,
  ) -> MidiManagerState {
    MidiManagerState { dst: BUS_DRY }
  }
}

impl Ugen for MidiManagerState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    true
  }
  fn release(&mut self) {}
  fn restrike(&mut self, _vel: f32) {}
}

use std::sync::Arc;

use crate::audio::BUF_SIZE;
use crate::consts::BUS_DRY;
use crate::envelope::Adsr;
use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen::GetSrcBuf;
use crate::ugen::Ugen;

#[derive(Clone, Debug)]
pub struct MidiManagerState {
  dst: usize,
  buf: Vec<f32>,
}

impl MidiManagerState {
  pub fn new(
    freq_hz: f32,
    freq2_hz: f32,
    adsr: Adsr,
    wavetable: Arc<Vec<f32>>,
  ) -> MidiManagerState {
    MidiManagerState {
      dst: BUS_DRY,
      buf: vec![0.; BUF_SIZE],
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

impl GetSrcBuf for MidiManagerState {
  fn get_src_buf(&self) -> &Vec<f32> {
    &self.buf
  }
}

use serde::{Deserialize, Serialize};

use crate::state::{ControlBlock, ControlBlocks, GenState};
use crate::ugen::Ugen;

const HISTORY_SIZE: usize = 35000;

#[derive(Clone, Debug)]
pub struct AllpassState {
  src: usize,
  dst: usize,
  ctl: usize,
  ix: usize,
  memory_rec: Vec<f32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
pub struct AllpassControlBlock {
  pub gain: f32,
  pub delay: usize,
  pub naive: bool,
}

impl AllpassState {
  pub fn new(src: usize, dst: usize, ctl: usize) -> Self {
    AllpassState {
      src,
      dst,
      ctl,
      ix: 0,
      memory_rec: vec![0.; HISTORY_SIZE],
    }
  }

  fn do_tap(&self, delay: i32) -> f32 {
    self.memory_rec[((self.ix as i32) - delay).rem_euclid(HISTORY_SIZE as i32) as usize]
  }

  fn ctl_run(&mut self, gen: GenState, tick_s: f32, ctl: &AllpassControlBlock) -> bool {
    for bus_ix in 0..gen.audio_bus[0].len() {
      // bus_ix is the index into the past output (memory_rec) of this
      // ugen.

      let AllpassControlBlock {
        gain: g,
        delay,
        naive,
      } = ctl;
      // Make sure the current input value is in memory_input at the
      // current time (because do_tap might need to read it)
      let dry = gen.audio_bus[self.src][bus_ix];
      // tapv = (Δ + gΔ² + g³Δ² + ⋯)dry
      let tapv = self.do_tap(*delay as i32);
      // wet = dry/(1-gΔ) = (1 + gΔ + (gΔ)² + (gΔ)³ + ⋯)dry
      let wet = dry + g * tapv;
      self.memory_rec[self.ix] = wet;

      // Schroeder & Logan 1960 "'Colorless' Artificial Reverberation"
      let out = if *naive {
        dry + g * tapv
      } else {
        -g * dry + (1.0 - g * g) * tapv
      };

      gen.audio_bus[self.dst][bus_ix] = out;
      self.ix = (self.ix + 1) % HISTORY_SIZE;
    }
    true
  }
}

impl Ugen for AllpassState {
  fn run(&mut self, gen: GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match &ctl[self.ctl] {
      Some(ControlBlock::All(ctl)) => self.ctl_run(gen, tick_s, ctl),
      _ => false,
    }
  }
}

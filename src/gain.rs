use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::state::{ControlBlock, ControlBlocks, GenState};
use crate::ugen::Ugen;

const LOW_PASS_AMOUNT: usize = 35000;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tap {
  pub pos: usize,
  pub weight: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub struct GainControlBlock {
  pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct GainState {
  src: usize,
  dst: usize,
  ci: usize,
}

impl GainState {
  pub fn new(src: usize, dst: usize, ci: usize) -> Self {
    GainState { src, dst, ci }
  }

  fn ctl_run(&mut self, gen: GenState, ctl: &GainControlBlock) -> bool {
    for bus_ix in 0..gen.audio_bus[0].len() {
      gen.audio_bus[self.dst][bus_ix] = gen.audio_bus[self.src][bus_ix] * ctl.scale;
    }
    true
  }
}

impl Ugen for GainState {
  fn run(&mut self, gen: GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match &ctl[self.ci] {
      Some(ControlBlock::Gain(ctl)) => self.ctl_run(gen, &ctl),
      _ => false,
    }
  }
}

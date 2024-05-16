use serde::{Deserialize, Serialize};

use crate::state::{ControlBlock, ControlBlocks, GenState};
use crate::ugen::Ugen;

const LOW_PASS_AMOUNT: usize = 35000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub struct Tap {
  pub pos: usize,
  pub weight: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
pub struct GainControlBlock {
  pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct GainState {
  src: usize,
  dst: usize,
}

impl GainState {
  pub fn new(src: usize, dst: usize) -> Self {
    GainState { src, dst }
  }

  fn ctl_run(&mut self, gen: &mut GenState, ctl: &GainControlBlock) -> bool {
    for bus_ix in 0..gen.audio_bus[0].len() {
      gen.audio_bus[self.dst][bus_ix] = gen.audio_bus[self.src][bus_ix] * ctl.scale;
    }
    true
  }
}

impl Ugen for GainState {
  fn run(
    &mut self,
    gen: &mut GenState,
    advice: &crate::ugen::Advice,
    tick_s: f32,
    ctl: &ControlBlocks,
  ) -> bool {
    // XXX hard coded index
    match &ctl[2] {
      Some(ControlBlock::Gain(ctl)) => self.ctl_run(gen, &ctl),
      _ => false,
    }
  }
}

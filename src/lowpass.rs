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
pub struct LowpassControlBlock {
  pub self_weight: f32,
  pub taps: Vec<Tap>,
}

#[derive(Clone, Debug)]
pub struct LowpassState {
  src: usize,
  dst: usize,
  ix: usize,
  memory: Vec<f32>,
}

impl LowpassState {
  pub fn new(src: usize, dst: usize) -> Self {
    LowpassState {
      src,
      dst,
      ix: 0,
      memory: vec![0.; LOW_PASS_AMOUNT],
    }
  }

  fn ctl_run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &LowpassControlBlock) -> bool {
    let len = self.memory.len();
    for bus_ix in 0..gen.audio_bus[0].len() {
      // advance

      let do_tap = |offset: i32, scale: f32| -> f32 {
        scale * self.memory[((self.ix as i32) - offset).rem_euclid(len as i32) as usize]
      };

      let mut wet = ctl.self_weight * gen.audio_bus[self.src][bus_ix];
      for tap in ctl.taps.iter() {
        wet += do_tap(tap.pos as i32, tap.weight);
      }

      gen.audio_bus[self.dst][bus_ix] = wet;
      self.ix = (self.ix + 1) % len;
      self.memory[self.ix] = wet;
    }
    true
  }
}

impl Ugen for LowpassState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    // XXX hard coded index
    match &ctl[1] {
      ControlBlock::Low(ctl) => self.ctl_run(gen, tick_s, &ctl),
      _ => false,
    }
  }
}

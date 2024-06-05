use serde::{Deserialize, Serialize};

use crate::state::{ControlBlock, ControlBlocks, GenState};
use crate::ugen::Ugen;
use rand::Rng;

const HISTORY_SIZE: usize = 35000;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum TapType {
  Rec,
  Input,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub struct Tap {
  pub tp: TapType,
  pub pos: usize,
  pub weight: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
pub struct LowpassControlBlock {
  pub taps: Vec<Tap>,
}

#[derive(Clone, Debug)]
pub struct LowpassState {
  src: usize,
  dst: usize,
  ix: usize,
  offset: i32,
  memory_rec: Vec<f32>,
  memory_input: Vec<f32>,
}

impl LowpassState {
  pub fn new(src: usize, dst: usize) -> Self {
    LowpassState {
      src,
      dst,
      offset: 0,
      ix: 0,
      memory_rec: vec![0.; HISTORY_SIZE],
      memory_input: vec![0.; HISTORY_SIZE],
    }
  }

  fn do_tap(&self, tap: &Tap, extra_pos: i32) -> f32 {
    let memory = match tap.tp {
      TapType::Input => &self.memory_input,
      TapType::Rec => &self.memory_rec,
    };
    let memval = memory
      [((self.ix as i32) - (tap.pos as i32) - extra_pos).rem_euclid(HISTORY_SIZE as i32) as usize];
    tap.weight * memval
  }

  fn ctl_run(&mut self, gen: GenState, tick_s: f32, ctl: &LowpassControlBlock) -> bool {
    for bus_ix in 0..gen.audio_bus[0].len() {
      self.offset = rand::thread_rng().gen_range(0..5);

      // bus_ix is the index into the snippet of audio we are
      // currently processing self.ix is the index into the ring
      // buffers that remember past input (memory_input), and past
      // output (memory_rec) of this ugen.

      // Make sure the current input value is in memory_input at the
      // current time (because do_tap might need to read it)
      self.memory_input[self.ix] = gen.audio_bus[self.src][bus_ix];
      let mut wet = 0.;
      for tap in ctl.taps.iter() {
        wet += self.do_tap(tap, self.offset);
      }
      gen.audio_bus[self.dst][bus_ix] = wet;
      self.memory_rec[self.ix] = wet;
      self.ix = (self.ix + 1) % HISTORY_SIZE;
    }
    true
  }
}

impl Ugen for LowpassState {
  fn run(&mut self, gen: GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    // XXX hard coded index
    match &ctl[1] {
      Some(ControlBlock::Low(ctl)) => self.ctl_run(gen, tick_s, &ctl),
      _ => false,
    }
  }
}

use crate::freeverb::Freeverb;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

use crate::state::{ControlBlock, ControlBlocks, GenState};
use crate::ugen::Ugen;

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
pub struct ReverbControlBlock {
  room_size: f32,
  wet: f32,
}

pub struct ReverbState {
  src: usize,
  dst: usize,
  freeverb_state: Freeverb,
}

impl Debug for ReverbState {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "---")
  }
}

impl ReverbState {
  pub fn new(src: usize, dst: usize) -> Self {
    let mut freeverb_state = Freeverb::new(44100);
    freeverb_state.set_room_size(0.2f64);
    freeverb_state.set_dry(0.9f64);
    freeverb_state.set_wet(0.1f64);
    ReverbState {
      src,
      dst,
      freeverb_state,
    }
  }

  fn ctl_run(&mut self, gen: GenState, tick_s: f32, ctl: &ReverbControlBlock) -> bool {
    let freeverb_state = &mut self.freeverb_state;
    freeverb_state.set_room_size((0.01 + 0.98 * ctl.room_size) as f64);
    freeverb_state.set_dry((1.0 - ctl.wet) as f64);
    freeverb_state.set_wet(ctl.wet as f64);

    for bus_ix in 0..gen.audio_bus[0].len() {
      let inv = gen.audio_bus[self.src][bus_ix];
      let (left, right) = self.freeverb_state.tick((inv as f64, inv as f64));
      gen.audio_bus[self.dst][bus_ix] = ((left + right) / 0.5) as f32;
    }
    true
  }
}

impl Ugen for ReverbState {
  fn run(&mut self, gen: GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    // XXX hard coded index
    match &ctl[4] {
      Some(ControlBlock::Reverb(ctl)) => self.ctl_run(gen, tick_s, &ctl),
      _ => false,
    }
  }
}

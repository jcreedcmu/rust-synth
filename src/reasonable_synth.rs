use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::consts::SAMPLE_RATE_hz;
use crate::envelope::{Adsr, EnvPos, EnvState};
use crate::state::{ControlBlocks, GenState};
use crate::ugen::Ugen;

const reasonable_adsr: Adsr = Adsr {
  attack_s: 0.001,
  decay_s: 0.005,
  sustain: 0.3,
  release_s: 0.05,
};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
pub struct ReasonableControlBlock {}

#[derive(Clone, Debug)]
pub struct ReasonableSynthState {
  dst: usize,
  freq_hz: f32,
  phase: f32,
  pub env_state: EnvState, // XXX make private?
  wavetable: Arc<Vec<f32>>,
}

impl ReasonableSynthState {
  pub fn new(dst: usize, freq_hz: f32, vel: f32, wavetable: Arc<Vec<f32>>) -> Self {
    ReasonableSynthState {
      dst,
      phase: 0.0,
      freq_hz,
      env_state: EnvState {
        adsr: reasonable_adsr,
        pos: EnvPos::On {
          amp: 0.0,
          t_s: 0.0,
          vel,
          hold: true,
        },
      },
      wavetable,
    }
  }
}

impl Ugen for ReasonableSynthState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    for out in gen.audio_bus[self.dst].iter_mut() {
      let table_phase: f32 = self.phase * ((self.wavetable.len() - 1) as f32);
      let offset = table_phase.floor() as usize;

      let fpart: f32 = (table_phase as f32) - (offset as f32);

      // linear interp
      // Without this conditional, ometimes we crash here because
      // offset is exactly self.wavetable.len() - 1
      let table_val = if offset + 1 >= self.wavetable.len() {
        self.wavetable[self.wavetable.len() - 1]
      } else {
        fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset]
      };

      let scale = self.env_state.amp();
      *out += (scale as f32) * table_val;

      // advance
      self.phase += self.freq_hz / SAMPLE_RATE_hz;
      if self.phase > 1. {
        self.phase -= 1.;
      }
      if !self.env_state.advance(tick_s) {
        return false;
      }
    }
    true
  }
}

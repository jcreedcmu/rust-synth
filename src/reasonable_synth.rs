use std::sync::Arc;

use crate::consts::{RELEASE_s, SAMPLE_RATE_hz};
use crate::envelope::EnvState;
use crate::ugen::Ugen;

#[derive(Clone, Debug)]
pub struct ReasonableSynthState {
  freq_hz: f32,
  phase: f32,
  env_state: EnvState,
  wavetable: Arc<Vec<f32>>,
}

impl ReasonableSynthState {
  pub fn new(freq_hz: f32, vel: f32, wavetable: Arc<Vec<f32>>) -> Self {
    ReasonableSynthState {
      phase: 0.0,
      freq_hz,
      env_state: EnvState::On {
        amp: 0.0,
        t_s: 0.0,
        vel,
      },
      wavetable,
    }
  }
}

impl Ugen for ReasonableSynthState {
  fn run(&self) -> f32 {
    let table_phase: f32 = self.phase * ((self.wavetable.len() - 1) as f32);
    let offset = table_phase.floor() as usize;

    let fpart: f32 = (table_phase as f32) - (offset as f32);

    // linear interp
    let table_val = fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset];

    let scale = self.env_state.amp();
    (scale as f32) * table_val
  }

  // returns true if should continue note
  fn advance(&mut self, tick_s: f32) -> bool {
    self.phase += self.freq_hz / SAMPLE_RATE_hz;
    if self.phase > 1. {
      self.phase -= 1.;
    }
    self.env_state.advance(tick_s)
  }

  fn release(&mut self) {
    self.env_state = EnvState::Release {
      t_s: 0.0,
      amp: self.env_state.amp(),
      dur_s: RELEASE_s,
    };
  }

  fn restrike(&mut self, vel: f32) {
    self.env_state = EnvState::On {
      t_s: 0.0,
      amp: self.env_state.amp(),
      vel,
    };
  }
}

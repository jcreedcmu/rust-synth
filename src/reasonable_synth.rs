use std::sync::Arc;

use crate::consts::{RELEASE_s, SAMPLE_RATE_hz};
use crate::state::EnvState;
use crate::synth::ugen_env_amp;
use crate::ugen::Ugen;

// Advance ugen state forward by tick_s
// returns true if we should terminate the ugen
pub fn advance_envelope(env: &mut EnvState, tick_s: f32) -> bool {
  match env {
    EnvState::On { ref mut t_s, .. } => {
      *t_s += tick_s;
      false
    },
    EnvState::Release { ref mut t_s, .. } => {
      *t_s += tick_s;
      *t_s > RELEASE_s
    },
  }
}

#[derive(Clone, Debug)]
pub struct ReasonableSynthState {
  freq_hz: f32,
  phase: f32,
  env_state: EnvState,
  wavetable: Arc<Vec<f32>>,
}

impl ReasonableSynthState {
  pub fn new(freq_hz: f32, vel: f32, wavetable: Arc<Vec<f32>>) -> ReasonableSynthState {
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

  pub fn restrike(self: &mut ReasonableSynthState, vel: f32) {
    self.env_state = EnvState::On {
      t_s: 0.0,
      amp: self.get_current_amp(),
      vel,
    };
  }

  pub fn release(self: &mut ReasonableSynthState) {
    self.env_state = EnvState::Release {
      t_s: 0.0,
      amp: self.get_current_amp(),
    };
  }

  fn get_current_amp(self: &ReasonableSynthState) -> f32 {
    ugen_env_amp(&self.env_state)
  }
}

impl Ugen for ReasonableSynthState {
  fn run(self: &ReasonableSynthState) -> f32 {
    let table_phase: f32 = self.phase * ((self.wavetable.len() - 1) as f32);
    let offset = table_phase.floor() as usize;

    let fpart: f32 = (table_phase as f32) - (offset as f32);

    // linear interp
    let table_val = fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset];

    let scale = ugen_env_amp(&self.env_state);
    (scale as f32) * table_val
  }

  // returns true if should continue note
  fn advance(self: &mut ReasonableSynthState, tick_s: f32) -> bool {
    let ReasonableSynthState {
      freq_hz,
      phase,
      env_state,
      ..
    } = self;
    if advance_envelope(env_state, tick_s) {
      false
    } else {
      *phase += *freq_hz / SAMPLE_RATE_hz;
      if *phase > 1. {
        *phase -= 1.;
      }
      true
    }
  }
}

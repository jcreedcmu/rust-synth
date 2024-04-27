use std::sync::Arc;

use crate::consts::{SAMPLE_RATE_hz, BUS_DRY};
use crate::envelope::{Adsr, EnvPos, EnvState};
use crate::ugen::{AudioBusses, Ugen};

const reasonable_adsr: Adsr = Adsr {
  attack_s: 0.001,
  decay_s: 0.005,
  sustain: 0.3,
  release_s: 0.05,
};

#[derive(Debug)]
pub struct ReasonableControlBlock {}

#[derive(Clone, Debug)]
pub struct ReasonableSynthState {
  dst: usize,
  freq_hz: f32,
  phase: f32,
  env_state: EnvState,
  wavetable: Arc<Vec<f32>>,
}

impl ReasonableSynthState {
  pub fn new(freq_hz: f32, vel: f32, wavetable: Arc<Vec<f32>>) -> Self {
    ReasonableSynthState {
      dst: BUS_DRY,
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
  type ControlBlock = ReasonableControlBlock;

  fn run(&self, bus: &mut AudioBusses) {
    let table_phase: f32 = self.phase * ((self.wavetable.len() - 1) as f32);
    let offset = table_phase.floor() as usize;

    let fpart: f32 = (table_phase as f32) - (offset as f32);

    // linear interp
    let table_val = fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset];

    let scale = self.env_state.amp();
    bus[self.dst] += (scale as f32) * table_val
  }

  // returns true if should continue note
  fn advance(&mut self, tick_s: f32, bus: &AudioBusses) -> bool {
    self.phase += self.freq_hz / SAMPLE_RATE_hz;
    if self.phase > 1. {
      self.phase -= 1.;
    }
    self.env_state.advance(tick_s)
  }

  fn release(&mut self) {
    self.env_state.pos = EnvPos::Release {
      t_s: 0.0,
      amp: self.env_state.amp(),
    };
  }

  fn restrike(&mut self, vel: f32) {
    self.env_state.pos = EnvPos::On {
      t_s: 0.0,
      amp: self.env_state.amp(),
      vel,
      hold: true,
    };
  }
}

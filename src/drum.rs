use std::sync::Arc;

use crate::envelope::EnvState;
use crate::synth::TABLE_SIZE;
use crate::{consts::SAMPLE_RATE_hz, ugen::Ugen};

const DRUM_DEBUG_RELEASE_s: f32 = 0.2;

#[derive(Clone, Debug)]
pub struct DrumSynthState {
  t_s: f32,
  freq_hz: f32,
  phase: f32,
  env_state: EnvState,
  wavetable: Arc<Vec<f32>>,
}

impl DrumSynthState {
  pub fn new(freq_hz: f32, wavetable: Arc<Vec<f32>>) -> DrumSynthState {
    DrumSynthState {
      t_s: 0.0,
      phase: 0.0,
      freq_hz,
      env_state: EnvState::Release {
        amp: 1.0,
        t_s: 0.0,
        dur_s: DRUM_DEBUG_RELEASE_s,
      },
      wavetable,
    }
  }
}

impl Ugen for DrumSynthState {
  fn run(self: &DrumSynthState) -> f32 {
    let table_phase: f32 = self.phase * ((self.wavetable.len() - 1) as f32);
    let offset = table_phase.floor() as usize;

    let fpart: f32 = (table_phase as f32) - (offset as f32);

    // linear interp
    let table_val = fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset];

    0.05 * table_val * self.env_state.amp()
  }

  // returns true if should continue note
  // returns true if should continue note
  fn advance(&mut self, tick_s: f32) -> bool {
    let drum_freq_hz: f32 = self.freq_hz / (TABLE_SIZE as f32);
    self.phase += drum_freq_hz / SAMPLE_RATE_hz;
    if self.phase > 1. {
      self.phase -= 1.;
    }
    self.env_state.advance(tick_s)
  }

  fn release(&mut self) {}
  fn restrike(&mut self, _vel: f32) {}
}

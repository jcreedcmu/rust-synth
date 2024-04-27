use std::sync::Arc;

use crate::consts::BUS_DRY;
use crate::envelope::{Adsr, EnvPos, EnvState};
use crate::state::{AudioBusses, ControlBlock, ControlBlocks};
use crate::synth::TABLE_SIZE;
use crate::{consts::SAMPLE_RATE_hz, ugen::Ugen};

const drum_adsr: Adsr = Adsr {
  attack_s: 0.025,
  decay_s: 0.05,
  sustain: 0.1,
  release_s: 0.25,
};

#[derive(Debug)]
pub struct DrumControlBlock {}

#[derive(Clone, Debug)]
pub struct DrumSynthState {
  dst: usize,
  t_s: f32,
  freq_hz: f32,
  freq2_hz: f32,
  phase: f32,
  vol: f32,
  env_state: EnvState,
  wavetable: Arc<Vec<f32>>,
  ci: usize,
}

impl DrumSynthState {
  pub fn new(freq_hz: f32, freq2_hz: f32, vol: f32, wavetable: Arc<Vec<f32>>) -> DrumSynthState {
    DrumSynthState {
      dst: BUS_DRY,
      t_s: 0.0,
      phase: 0.0,
      vol,
      freq_hz,
      freq2_hz,
      env_state: EnvState {
        pos: EnvPos::On {
          amp: 0.0,
          t_s: 0.0,
          hold: false,
          vel: 1.0,
        },
        adsr: drum_adsr,
      },
      wavetable,
      ci: 0,
    }
  }

  fn ctl_run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &DrumControlBlock) -> bool {
    for out in bus[self.dst].iter_mut() {
      let table_phase: f32 = self.phase * ((self.wavetable.len() - 1) as f32);
      let offset = table_phase.floor() as usize;

      let fpart: f32 = (table_phase as f32) - (offset as f32);

      // linear interp
      let table_val = fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset];

      *out += 0.15 * table_val * self.env_state.amp() * self.vol;

      // advance
      let a = self.env_state.time_s() / self.env_state.attack_len_s();
      let eff_freq_hz = a * self.freq2_hz + (1.0 - a) * self.freq_hz;
      let drum_freq_hz: f32 = eff_freq_hz / (TABLE_SIZE as f32);
      self.phase += drum_freq_hz / SAMPLE_RATE_hz;
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

impl Ugen for DrumSynthState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match &ctl[self.ci] {
      ControlBlock::Drum(ctl) => self.ctl_run(bus, tick_s, &ctl),
      _ => false,
    }
  }

  fn release(&mut self) {}
  fn restrike(&mut self, _vel: f32) {}
}

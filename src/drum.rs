use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::consts::BUS_DRY;
use crate::envelope::{Adsr, EnvState};
use crate::state::{ControlBlock, ControlBlocks, GenState};
use crate::synth::TABLE_SIZE;
use crate::{consts::SAMPLE_RATE_hz, ugen::Ugen};

pub fn drum_adsr(dur_scale: f32) -> Adsr {
  Adsr {
    attack_s: 0.01 * dur_scale,
    decay_s: 0.05 * dur_scale,
    sustain: 0.2,
    release_s: 0.2 * dur_scale,
  }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
pub struct DrumControlBlock {
  pub vol: f32,
  pub freq_hz: f32,
  pub freq2_hz: f32,
  pub adsr: Adsr,
}

#[derive(Clone, Debug)]
pub struct DrumSynthState {
  dst: usize,
  t_s: f32,
  phase: f32,
  env_state: EnvState,
  wavetable: Arc<Vec<f32>>,
  ci: usize,
}

impl DrumSynthState {
  pub fn new(wavetable: Arc<Vec<f32>>, ci: usize) -> DrumSynthState {
    DrumSynthState {
      dst: BUS_DRY,
      t_s: 0.0,
      phase: 0.0,
      env_state: EnvState::On {
        amp: 0.0,
        t_s: 0.0,
        hold: false,
        vel: 1.0,
      },
      wavetable,
      ci,
    }
  }

  fn ctl_run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &DrumControlBlock) -> bool {
    let DrumControlBlock { adsr, .. } = ctl;
    for out in gen.audio_bus[self.dst].iter_mut() {
      let table_phase: f32 = self.phase * ((self.wavetable.len() - 1) as f32);
      let offset = table_phase.floor() as usize;

      let fpart: f32 = (table_phase as f32) - (offset as f32);

      // linear interp
      let table_val = fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset];

      *out += 0.15 * table_val * self.env_state.amp(adsr) * ctl.vol;

      // advance
      let a = self.env_state.time_s(adsr) / adsr.attack_len_s();
      let eff_freq_hz = a * ctl.freq2_hz + (1.0 - a) * ctl.freq_hz;
      let drum_freq_hz: f32 = eff_freq_hz / (TABLE_SIZE as f32);
      self.phase += drum_freq_hz / SAMPLE_RATE_hz;
      if self.phase > 1. {
        self.phase -= 1.;
      }
      if !self.env_state.advance(tick_s, adsr) {
        return false;
      }
    }
    true
  }
}

impl Ugen for DrumSynthState {
  fn run(
    &mut self,
    gen: &mut GenState,
    advice: &crate::ugen::Advice,
    tick_s: f32,
    ctl: &ControlBlocks,
  ) -> bool {
    match &ctl[self.ci] {
      Some(ControlBlock::Drum(ctl)) => self.ctl_run(gen, tick_s, &ctl),
      _ => false,
    }
  }
}

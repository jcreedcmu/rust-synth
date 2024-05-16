use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::consts::SAMPLE_RATE_hz;
use crate::envelope::{Adsr, EnvState};
use crate::notegen::NoteMode;
use crate::state::{ControlBlock, ControlBlocks, GenState};
use crate::ugen::{Advice, Ugen};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
pub struct ReasonableControlBlock {
  pub adsr: Adsr, // XXX make private?
}

#[derive(Clone, Debug)]
pub struct ReasonableSynthState {
  dst: usize,
  freq_hz: f32,
  phase: f32,
  pub env_state: EnvState, // XXX make private?
  wavetable: Arc<Vec<f32>>,
  pub ci: usize, // XXX make private?
}

impl ReasonableSynthState {
  pub fn new(dst: usize, freq_hz: f32, vel: f32, wavetable: Arc<Vec<f32>>, ci: usize) -> Self {
    ReasonableSynthState {
      dst,
      phase: 0.0,
      freq_hz,
      env_state: EnvState::On {
        amp: 0.0,
        t_s: 0.0,
        vel,
        hold: true,
      },
      wavetable,
      ci,
    }
  }

  fn ctl_run(
    &mut self,
    gen: &mut GenState,
    advice: &Advice,
    tick_s: f32,
    ctl: &ReasonableControlBlock,
  ) -> bool {
    let ReasonableControlBlock { adsr } = ctl;
    let Advice { note_mode } = advice;

    match note_mode {
      NoteMode::Release => {
        self.env_state = EnvState::Release {
          t_s: 0.0,
          amp: self.env_state.amp(adsr),
        };
      },
      NoteMode::Restrike { vel } => {
        self.env_state = EnvState::On {
          t_s: 0.0,
          amp: self.env_state.amp(adsr),
          vel: *vel,
          hold: true,
        };
      },
      NoteMode::Run => (),
    }

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

      let scale = self.env_state.amp(adsr);
      *out += (scale as f32) * table_val;

      // advance
      self.phase += self.freq_hz / SAMPLE_RATE_hz;
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

impl Ugen for ReasonableSynthState {
  fn run(
    &mut self,
    gen: &mut GenState,
    advice: &crate::ugen::Advice,
    tick_s: f32,
    ctl: &ControlBlocks,
  ) -> bool {
    match &ctl[self.ci] {
      Some(ControlBlock::Reasonable(ctl)) => self.ctl_run(gen, advice, tick_s, &ctl),
      _ => false,
    }
  }
}

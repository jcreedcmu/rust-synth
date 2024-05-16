use std::sync::Arc;

use crate::envelope::EnvPos;
use crate::reasonable_synth::ReasonableSynthState;
use crate::state::{ControlBlocks, GenState};
use crate::ugen::Ugen;

#[derive(Debug)]
enum NoteMode {
  Run,
  Release,
  Restrike { vel: f32 },
}

pub trait Notegen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool;
  fn release(&mut self);
  fn restrike(&mut self, vel: f32);
}

#[derive(Debug)]
pub struct NotegenState {
  mode: NoteMode,
  ugen: ReasonableSynthState,
}

// some boilerplate to wire things up
impl Notegen for NotegenState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self.mode {
      NoteMode::Release => {
        self.ugen.env_state.pos = EnvPos::Release {
          t_s: 0.0,
          amp: self.ugen.env_state.amp(),
        };
      },
      NoteMode::Restrike { vel } => {
        self.ugen.env_state.pos = EnvPos::On {
          t_s: 0.0,
          amp: self.ugen.env_state.amp(),
          vel,
          hold: true,
        };
      },
      NoteMode::Run => (),
    }
    self.mode = NoteMode::Run;
    self.ugen.run(gen, tick_s, ctl)
  }

  fn release(&mut self) {
    self.mode = NoteMode::Release;
  }

  fn restrike(&mut self, vel: f32) {
    self.mode = NoteMode::Restrike { vel };
  }
}

impl NotegenState {
  pub fn new(dst: usize, freq_hz: f32, vel: f32, wavetable: Arc<Vec<f32>>) -> Self {
    NotegenState {
      ugen: ReasonableSynthState::new(dst, freq_hz, vel, wavetable),
      mode: NoteMode::Run,
    }
  }
}
pub type NotegensState = Vec<Option<NotegenState>>;

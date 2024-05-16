use std::sync::Arc;

use crate::envelope::EnvState;
use crate::reasonable_synth::{ReasonableControlBlock, ReasonableSynthState};
use crate::state::{ControlBlock, ControlBlocks, GenState};

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

impl NotegenState {
  fn ctl_run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ReasonableControlBlock) -> bool {
    let ReasonableControlBlock { adsr, .. } = ctl;
    match self.mode {
      NoteMode::Release => {
        self.ugen.env_state = EnvState::Release {
          t_s: 0.0,
          amp: self.ugen.env_state.amp(adsr),
        };
      },
      NoteMode::Restrike { vel } => {
        self.ugen.env_state = EnvState::On {
          t_s: 0.0,
          amp: self.ugen.env_state.amp(adsr),
          vel,
          hold: true,
        };
      },
      NoteMode::Run => (),
    }
    self.mode = NoteMode::Run;
    self.ugen.ctl_run(gen, tick_s, ctl)
  }
}

// some boilerplate to wire things up
impl Notegen for NotegenState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match &ctl[self.ugen.ci] {
      Some(ControlBlock::Reasonable(ctl)) => self.ctl_run(gen, tick_s, &ctl),
      _ => false,
    }
  }

  fn release(&mut self) {
    self.mode = NoteMode::Release;
  }

  fn restrike(&mut self, vel: f32) {
    self.mode = NoteMode::Restrike { vel };
  }
}

impl NotegenState {
  pub fn new(dst: usize, freq_hz: f32, vel: f32, wavetable: Arc<Vec<f32>>, ci: usize) -> Self {
    NotegenState {
      ugen: ReasonableSynthState::new(dst, freq_hz, vel, wavetable, ci),
      mode: NoteMode::Run,
    }
  }
}
pub type NotegensState = Vec<Option<NotegenState>>;

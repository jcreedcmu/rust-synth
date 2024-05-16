use std::mem;
use std::sync::Arc;

use crate::reasonable_synth::ReasonableSynthState;
use crate::state::{ControlBlocks, GenState};
use crate::ugen::{Advice, Ugen};

#[derive(Debug)]
pub enum NoteMode {
  Run,
  Release,
  Restrike { vel: f32 },
}

pub trait Notegen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, gen: &mut GenState, advice: &Advice, tick_s: f32, ctl: &ControlBlocks) -> bool;
  fn release(&mut self);
  fn restrike(&mut self, vel: f32);
}

#[derive(Debug)]
pub struct NotegenState {
  mode: NoteMode,
  ugen: ReasonableSynthState,
}

impl Notegen for NotegenState {
  fn run(&mut self, gen: &mut GenState, advice: &Advice, tick_s: f32, ctl: &ControlBlocks) -> bool {
    // Maybe this is unnecessarily tricky, but what really needs to happen here is merely:
    // - replace advice.note_mode with self.note_mote before calling run
    // - set self.note_mode = NoteMode::Run
    // and this is one way of accomplishing that.
    let mut note_mode = NoteMode::Run;
    mem::swap(&mut self.mode, &mut note_mode);
    let advice = Advice { note_mode };
    self.ugen.run(gen, &advice, tick_s, &ctl)
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

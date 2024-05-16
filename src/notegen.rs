use std::mem;

use crate::state::{ControlBlocks, GenState};
use crate::ugen::{Advice, Ugen, UgenState};

#[derive(Debug)]
pub enum NoteMode {
  Run,
  Release,
  Restrike { vel: f32 },
}

#[derive(Debug)]
pub struct NotegenState {
  mode: NoteMode,
  ugen: UgenState,
}

impl NotegenState {
  pub fn run(
    &mut self,
    mut gen: GenState,
    advice: &Advice,
    tick_s: f32,
    ctl: &ControlBlocks,
  ) -> bool {
    // Maybe this is unnecessarily tricky, but what really needs to happen here is merely:
    // - replace advice.note_mode with self.note_mote before calling run
    // - set self.note_mode = NoteMode::Run
    // and this is one way of accomplishing that.
    let mut note_mode = NoteMode::Run;
    mem::swap(&mut self.mode, &mut note_mode);
    let advice = Advice { note_mode };
    self.ugen.run(gen.readvise(&advice), &advice, tick_s, &ctl)
  }

  pub fn release(&mut self) {
    self.mode = NoteMode::Release;
  }

  pub fn restrike(&mut self, vel: f32) {
    self.mode = NoteMode::Restrike { vel };
  }

  pub fn new(ugen: UgenState) -> Self {
    NotegenState {
      ugen,
      mode: NoteMode::Run,
    }
  }
}
pub type NotegensState = Vec<Option<NotegenState>>;

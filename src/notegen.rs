use crate::reasonable_synth::ReasonableSynthState;
use crate::state::{ControlBlocks, GenState};

pub trait Notegen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool;
  fn release(&mut self);
  fn restrike(&mut self, vel: f32);
}

#[derive(Debug)]
pub enum NotegenState {
  ReasonableSynth(ReasonableSynthState),
}

// some boilerplate to wire things up
impl Notegen for NotegenState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self {
      NotegenState::ReasonableSynth(s) => s.run(gen, tick_s, ctl),
    }
  }

  fn release(&mut self) {
    match self {
      NotegenState::ReasonableSynth(s) => s.release(),
    }
  }

  fn restrike(&mut self, vel: f32) {
    match self {
      NotegenState::ReasonableSynth(s) => s.restrike(vel),
    }
  }
}

pub type NotegensState = Vec<Option<NotegenState>>;

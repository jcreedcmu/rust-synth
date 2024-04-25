use crate::drum::DrumSynthState;
use crate::reasonable_synth::ReasonableSynthState;

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&self) -> f32;
  fn advance(&mut self, tick_s: f32) -> bool;
  fn release(&mut self);
  fn restrike(&mut self, vel: f32);
}

#[derive(Debug)]
pub enum UgenState {
  ReasonableSynth(ReasonableSynthState),
  DrumSynth(DrumSynthState),
}

// some boilerplate to wire things up
impl Ugen for UgenState {
  fn run(&self) -> f32 {
    match self {
      UgenState::DrumSynth(s) => s.run(),
      UgenState::ReasonableSynth(s) => s.run(),
    }
  }

  fn advance(&mut self, tick_s: f32) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.advance(tick_s),
      UgenState::ReasonableSynth(s) => s.advance(tick_s),
    }
  }

  fn release(&mut self) {
    match self {
      UgenState::DrumSynth(s) => s.release(),
      UgenState::ReasonableSynth(s) => s.release(),
    }
  }

  fn restrike(&mut self, vel: f32) {
    match self {
      UgenState::DrumSynth(s) => s.restrike(vel),
      UgenState::ReasonableSynth(s) => s.restrike(vel),
    }
  }
}

pub type UgensState = Vec<Option<UgenState>>;

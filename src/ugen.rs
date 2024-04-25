use crate::drum::DrumSynthState;
use crate::lowpass::LowpassState;
use crate::reasonable_synth::ReasonableSynthState;

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&self, bus: &mut Vec<f32>);
  fn advance(&mut self, tick_s: f32, bus: &Vec<f32>) -> bool;
  fn release(&mut self);
  fn restrike(&mut self, vel: f32);
}

#[derive(Debug)]
pub enum UgenState {
  ReasonableSynth(ReasonableSynthState),
  DrumSynth(DrumSynthState),
  Lowpass(LowpassState),
}

// some boilerplate to wire things up
impl Ugen for UgenState {
  fn run(&self, bus: &mut Vec<f32>) {
    match self {
      UgenState::DrumSynth(s) => s.run(bus),
      UgenState::ReasonableSynth(s) => s.run(bus),
      UgenState::Lowpass(s) => s.run(bus),
    }
  }

  fn advance(&mut self, tick_s: f32, bus: &Vec<f32>) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.advance(tick_s, bus),
      UgenState::ReasonableSynth(s) => s.advance(tick_s, bus),
      UgenState::Lowpass(s) => s.advance(tick_s, bus),
    }
  }

  fn release(&mut self) {
    match self {
      UgenState::DrumSynth(s) => s.release(),
      UgenState::ReasonableSynth(s) => s.release(),
      UgenState::Lowpass(s) => s.release(),
    }
  }

  fn restrike(&mut self, vel: f32) {
    match self {
      UgenState::DrumSynth(s) => s.restrike(vel),
      UgenState::ReasonableSynth(s) => s.restrike(vel),
      UgenState::Lowpass(s) => s.restrike(vel),
    }
  }
}

pub type UgensState = Vec<Option<UgenState>>;

use crate::drum::DrumSynthState;
use crate::lowpass::LowpassState;
use crate::reasonable_synth::ReasonableSynthState;
use crate::state::{AudioBusses, ControlBlocks};

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool;
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
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.run(bus, tick_s, ctl),
      UgenState::ReasonableSynth(s) => s.run(bus, tick_s, ctl),
      UgenState::Lowpass(s) => s.run(bus, tick_s, ctl),
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

use crate::drum::DrumSynthState;
use crate::lowpass::LowpassState;
use crate::midi_manager::MidiManagerState;
use crate::state::{AudioBusses, ControlBlocks};

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool;
}

#[derive(Debug)]
pub enum UgenState {
  DrumSynth(DrumSynthState),
  Lowpass(LowpassState),
  MidiManager(MidiManagerState),
}

// some boilerplate to wire things up
impl Ugen for UgenState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.run(bus, tick_s, ctl),
      UgenState::Lowpass(s) => s.run(bus, tick_s, ctl),
      UgenState::MidiManager(s) => s.run(bus, tick_s, ctl),
    }
  }
}

pub type UgensState = Vec<Option<UgenState>>;

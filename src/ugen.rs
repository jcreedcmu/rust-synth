use serde::{Deserialize, Serialize};

use crate::drum::DrumSynthState;
use crate::lowpass::LowpassState;
use crate::midi_manager::MidiManagerState;
use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen_group::UgenGroupState;

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
pub enum UgenSpec {
  LowPass { src: usize, dst: usize },
  MidiManager { dst: usize },
  UgenGroup { dst: usize },
}

#[derive(Debug)]
pub enum UgenState {
  DrumSynth(DrumSynthState),
  Lowpass(LowpassState),
  MidiManager(MidiManagerState),
  UgenGroup(UgenGroupState),
}

// some boilerplate to wire things up
impl Ugen for UgenState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.run(bus, tick_s, ctl),
      UgenState::Lowpass(s) => s.run(bus, tick_s, ctl),
      UgenState::MidiManager(s) => s.run(bus, tick_s, ctl),
      UgenState::UgenGroup(s) => s.run(bus, tick_s, ctl),
    }
  }
}

impl UgenState {
  pub fn new(spec: UgenSpec) -> Self {
    match spec {
      UgenSpec::LowPass { src, dst } => UgenState::Lowpass(LowpassState::new(src, dst)),
      UgenSpec::MidiManager { dst } => UgenState::MidiManager(MidiManagerState::new(dst)),
      UgenSpec::UgenGroup { dst } => UgenState::UgenGroup(UgenGroupState::new(dst)),
    }
  }
}

// XXX make this not option
pub type UgensState = Vec<Option<UgenState>>;

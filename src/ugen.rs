use serde::{Deserialize, Serialize};

use crate::drum::DrumSynthState;
use crate::lowpass::LowpassState;
use crate::meter::MeterState;
use crate::midi_manager::MidiManagerState;
use crate::state::{ControlBlocks, GenState};
use crate::ugen_group::UgenGroupState;

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
pub enum UgenSpec {
  LowPass { src: usize, dst: usize },
  MidiManager { dst: usize },
  UgenGroup { dst: usize },
  Meter { src: usize },
}

#[derive(Debug)]
pub enum UgenState {
  DrumSynth(DrumSynthState),
  Lowpass(LowpassState),
  MidiManager(MidiManagerState),
  UgenGroup(UgenGroupState),
  Meter(MeterState),
}

// some boilerplate to wire things up
impl Ugen for UgenState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.run(gen, tick_s, ctl),
      UgenState::Lowpass(s) => s.run(gen, tick_s, ctl),
      UgenState::MidiManager(s) => s.run(gen, tick_s, ctl),
      UgenState::UgenGroup(s) => s.run(gen, tick_s, ctl),
      UgenState::Meter(s) => s.run(gen, tick_s, ctl),
    }
  }
}

impl UgenState {
  pub fn new(spec: UgenSpec) -> Self {
    match spec {
      UgenSpec::LowPass { src, dst } => UgenState::Lowpass(LowpassState::new(src, dst)),
      UgenSpec::MidiManager { dst } => UgenState::MidiManager(MidiManagerState::new(dst)),
      UgenSpec::UgenGroup { dst } => UgenState::UgenGroup(UgenGroupState::new(dst)),
      UgenSpec::Meter { src } => UgenState::Meter(MeterState::new(src)),
    }
  }
}

pub type UgensState = Vec<UgenState>;

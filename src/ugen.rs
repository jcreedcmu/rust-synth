use serde::{Deserialize, Serialize};

use crate::allpass::AllpassState;
use crate::drum::DrumSynthState;
use crate::gain::GainState;
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
  AllPass { src: usize, dst: usize, ctl: usize },
  MidiManager { dst: usize },
  UgenGroup { dst: usize },
  Meter { src: usize },
  Gain { src: usize, dst: usize },
}

#[derive(Debug)]
pub enum UgenState {
  DrumSynth(DrumSynthState),
  Lowpass(LowpassState),
  Allpass(AllpassState),
  MidiManager(MidiManagerState),
  UgenGroup(UgenGroupState),
  Meter(MeterState),
  Gain(GainState),
}

// some boilerplate to wire things up
impl Ugen for UgenState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.run(gen, tick_s, ctl),
      UgenState::Lowpass(s) => s.run(gen, tick_s, ctl),
      UgenState::Allpass(s) => s.run(gen, tick_s, ctl),
      UgenState::MidiManager(s) => s.run(gen, tick_s, ctl),
      UgenState::UgenGroup(s) => s.run(gen, tick_s, ctl),
      UgenState::Meter(s) => s.run(gen, tick_s, ctl),
      UgenState::Gain(s) => s.run(gen, tick_s, ctl),
    }
  }
}

impl UgenState {
  pub fn new(spec: UgenSpec) -> Self {
    match spec {
      UgenSpec::LowPass { src, dst } => UgenState::Lowpass(LowpassState::new(src, dst)),
      UgenSpec::AllPass { src, dst, ctl } => UgenState::Allpass(AllpassState::new(src, dst, ctl)),
      UgenSpec::MidiManager { dst } => UgenState::MidiManager(MidiManagerState::new(dst)),
      UgenSpec::UgenGroup { dst } => UgenState::UgenGroup(UgenGroupState::new(dst)),
      UgenSpec::Meter { src } => UgenState::Meter(MeterState::new(src)),
      UgenSpec::Gain { src, dst } => UgenState::Gain(GainState::new(src, dst)),
    }
  }
}

pub type UgensState = Vec<UgenState>;

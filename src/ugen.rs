use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::allpass::AllpassState;
use crate::drum::DrumSynthState;
use crate::gain::GainState;
use crate::lowpass::LowpassState;
use crate::meter::MeterState;
use crate::midi_manager::MidiManagerState;
use crate::notegen::NoteMode;
use crate::reasonable_synth::ReasonableSynthState;
use crate::reverb::ReverbState;
use crate::state::{ControlBlocks, GenState};
use crate::ugen_group::UgenGroupState;

#[derive(Debug)]
pub struct Advice {
  pub note_mode: NoteMode,
}

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&mut self, gen: GenState, tick_s: f32, ctl: &ControlBlocks) -> bool;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
#[serde(rename_all = "camelCase")]
#[derive(TS)]
#[ts(export)]
pub enum UgenSpec {
  LowPass { src: usize, dst: usize, ci: usize },
  AllPass { src: usize, dst: usize, ci: usize },
  MidiManager { dst: usize, ci: usize },
  UgenGroup { dst: usize },
  Meter { src: usize },
  Gain { src: usize, dst: usize, ci: usize },
  Reverb { src: usize, dst: usize, ci: usize },
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
  ReasonableSynth(ReasonableSynthState),
  Reverb(ReverbState),
}

// some boilerplate to wire things up
impl Ugen for UgenState {
  fn run(&mut self, gen: GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match self {
      UgenState::DrumSynth(s) => s.run(gen, tick_s, ctl),
      UgenState::Lowpass(s) => s.run(gen, tick_s, ctl),
      UgenState::Allpass(s) => s.run(gen, tick_s, ctl),
      UgenState::MidiManager(s) => s.run(gen, tick_s, ctl),
      UgenState::UgenGroup(s) => s.run(gen, tick_s, ctl),
      UgenState::Meter(s) => s.run(gen, tick_s, ctl),
      UgenState::Gain(s) => s.run(gen, tick_s, ctl),
      UgenState::ReasonableSynth(s) => s.run(gen, tick_s, ctl),
      UgenState::Reverb(s) => s.run(gen, tick_s, ctl),
    }
  }
}

impl UgenState {
  pub fn new(spec: UgenSpec) -> Self {
    match spec {
      UgenSpec::LowPass { src, dst, ci } => UgenState::Lowpass(LowpassState::new(src, dst, ci)),
      UgenSpec::AllPass { src, dst, ci } => UgenState::Allpass(AllpassState::new(src, dst, ci)),
      UgenSpec::MidiManager { dst, ci } => UgenState::MidiManager(MidiManagerState::new(dst, ci)),
      UgenSpec::UgenGroup { dst } => UgenState::UgenGroup(UgenGroupState::new(dst)),
      UgenSpec::Meter { src } => UgenState::Meter(MeterState::new(src)),
      UgenSpec::Gain { src, dst, ci } => UgenState::Gain(GainState::new(src, dst, ci)),
      UgenSpec::Reverb { src, dst, ci } => UgenState::Reverb(ReverbState::new(src, dst, ci)),
    }
  }
}

pub type UgensState = Vec<UgenState>;

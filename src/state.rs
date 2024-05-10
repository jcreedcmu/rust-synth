use std::sync::{Arc, Mutex};

use crate::consts::{AUDIO_BUS_LENGTH, BOTTOM_NOTE};
use crate::drum::{DrumControlBlock, DrumSynthState};
use crate::envelope::Adsr;
use crate::lowpass::LowpassControlBlock;
use crate::notegen::NotegenState;
use crate::reasonable_synth::{ReasonableControlBlock, ReasonableSynthState};
use crate::sequencer::Sequencer;
use crate::ugen::{UgenState, UgensState};
use crate::wavetables::Wavetables;
use crate::webserver::SynthMessage;

// XXX move to midi manager or reduce
#[derive(Clone, Debug)]
pub enum KeyState {
  Off,
  On { ugen_ix: usize },   // index into ugen_state vector
  Held { ugen_ix: usize }, // only on because pedal held
}

#[derive(Debug)]
pub enum ControlBlock {
  Reasonable(ReasonableControlBlock),
  Drum(DrumControlBlock),
  Low(LowpassControlBlock),
}

pub type ControlBlocks = Vec<ControlBlock>;

/// Outer vector is list of different busses. Inner vectors each
/// contain one monophonic buffer's worth of audio on
/// each bus.
pub type AudioBusses = Vec<Vec<f32>>;

#[derive(Debug)]
pub struct State {
  pub going: bool,

  // audio bus
  pub audio_bus: AudioBusses,

  // This has a varying length as synthesis goes on. Every time we
  // need to allocate a ugen, we try to reuse existing `None`s, but
  // push a new one if necessary.

  // I'm trying to move the midi-spawned notes into midi_manager, but
  // the sequencer-spawned drums can stay here for now.
  pub ugen_state: UgensState,

  // Effects go here
  pub fixed_ugens: UgensState,

  pub write_to_file: bool,

  pub control_blocks: ControlBlocks,
  pub wavetables: Wavetables,
  pub sequencer: Sequencer,
  pub websocket: Option<tokio::sync::mpsc::Sender<SynthMessage>>,
}

pub type StateGuard = Arc<Mutex<State>>;

pub const DEFAULT_DRUM_CONTROL_BLOCK: usize = 0;

impl State {
  pub fn new(buf_size: usize) -> State {
    let mut control_blocks: ControlBlocks = vec![];
    control_blocks.push(ControlBlock::Drum(DrumControlBlock { vol: 1. }));
    control_blocks.push(ControlBlock::Low(LowpassControlBlock { lowp_param: 0.5 }));
    State {
      going: true,
      audio_bus: vec![vec![0.; buf_size]; AUDIO_BUS_LENGTH],
      sequencer: Sequencer::new(),
      ugen_state: vec![],
      fixed_ugens: vec![],
      control_blocks,
      write_to_file: true,
      wavetables: Wavetables::new(),
      websocket: None,
    }
  }

  // XXX move to midi manager somehow?
  pub fn new_drum(&self, freq_hz: f32, freq2_hz: f32, adsr: Adsr) -> UgenState {
    UgenState::DrumSynth(DrumSynthState::new(
      freq_hz,
      freq2_hz,
      adsr,
      self.wavetables.noise_wavetable.clone(),
    ))
  }
}

// XXX move to MIDI manager maybe?

pub fn new_reasonable_of_tables(
  dst: usize,
  wavetables: &Wavetables,
  freq_hz: f32,
  vel: f32,
) -> NotegenState {
  NotegenState::ReasonableSynth(ReasonableSynthState::new(
    dst,
    freq_hz,
    vel,
    wavetables.saw_wavetable.clone(),
  ))
}

// XXX move to MIDI manager maybe?

pub fn get_key_state_mut(key_state: &mut Vec<KeyState>, pitch: usize) -> &mut KeyState {
  &mut key_state[pitch - (BOTTOM_NOTE as usize)]
}

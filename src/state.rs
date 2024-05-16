use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::allpass::AllpassControlBlock;
use crate::consts::{AUDIO_BUS_LENGTH, BOTTOM_NOTE};
use crate::drum::DrumControlBlock;
use crate::gain::GainControlBlock;
use crate::lowpass::LowpassControlBlock;
use crate::notegen::NotegenState;
use crate::reasonable_synth::{ReasonableControlBlock, ReasonableSynthState};
use crate::sequencer::Sequencer;
use crate::ugen::{Advice, UgenState, UgensState};
use crate::wavetables::Wavetables;
use crate::webserver::SynthMessage;

// XXX move to midi manager or reduce
#[derive(Clone, Debug)]
pub enum KeyState {
  Off,
  On { ugen_ix: usize },   // index into ugen_state vector
  Held { ugen_ix: usize }, // only on because pedal held
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "t")]
pub enum ControlBlock {
  Reasonable(ReasonableControlBlock),
  Drum(DrumControlBlock),
  Low(LowpassControlBlock),
  All(AllpassControlBlock),
  Gain(GainControlBlock),
}

pub type ControlBlocks = Vec<Option<ControlBlock>>;

/// Outer vector is list of different busses. Inner vectors each
/// contain one monophonic buffer's worth of audio on
/// each bus.
pub type AudioBusses = Vec<Vec<f32>>;

#[derive(Debug)]
pub struct GenState<'a> {
  pub audio_bus: &'a mut AudioBusses,
  pub websocket: &'a mut Option<tokio::sync::mpsc::Sender<SynthMessage>>,
  pub advice: &'a Advice,
}

impl<'a> GenState<'a> {
  /*
   * Issue: a bundle of mutable references does not automatically
   * reborrow, e.g. in loops. I'm following the model of
   * https://dwrensha.github.io/capnproto-rust/2014/12/27/custom-mutable-references.html
   * to solve this.
   */
  pub fn reborrow<'b>(&'b mut self) -> GenState<'b> {
    GenState {
      audio_bus: self.audio_bus,
      websocket: self.websocket,
      advice: self.advice,
    }
  }

  pub fn readvise<'b>(&'b mut self, advice: &'b Advice) -> GenState<'b> {
    GenState {
      advice,
      ..self.reborrow()
    }
  }
}

#[derive(Debug)]
pub struct State {
  pub going: bool,

  // audio bus
  pub audio_bus: AudioBusses,
  pub websocket: Option<tokio::sync::mpsc::Sender<SynthMessage>>,

  pub fixed_ugens: UgensState,

  pub write_to_file: bool,

  pub control_blocks: ControlBlocks,
  pub wavetables: Wavetables,
  pub sequencer: Sequencer,
}

pub type StateGuard = Arc<Mutex<State>>;

pub const DEFAULT_DRUM_CONTROL_BLOCK: usize = 10;
pub const DEFAULT_REASONABLE_CONTROL_BLOCK: usize = 0;
pub const DEFAULT_LOW_PASS_CONTROL_BLOCK: usize = 1;
pub const DEFAULT_GAIN_CONTROL_BLOCK: usize = 2;
pub const DEFAULT_ALL_PASS_CONTROL_BLOCK: usize = 3;
pub const NUM_CONTROL_BLOCKS: usize = 16;

impl State {
  pub fn new(buf_size: usize) -> State {
    let mut control_blocks: ControlBlocks = Vec::with_capacity(NUM_CONTROL_BLOCKS);
    control_blocks.resize_with(NUM_CONTROL_BLOCKS, || None);

    control_blocks[DEFAULT_GAIN_CONTROL_BLOCK] =
      Some(ControlBlock::Gain(GainControlBlock { scale: 1.0 }));
    control_blocks[DEFAULT_ALL_PASS_CONTROL_BLOCK] = Some(ControlBlock::All(AllpassControlBlock {
      delay: 10,
      gain: 0.7,
      naive: true,
    }));
    State {
      going: true,
      sequencer: Sequencer::new(),
      fixed_ugens: vec![],
      control_blocks,
      write_to_file: true,
      wavetables: Wavetables::new(),
      audio_bus: vec![vec![0.; buf_size]; AUDIO_BUS_LENGTH],
      websocket: None,
    }
  }

  // XXX move to midi manager somehow?
  pub fn new_drum(&self, ctl: usize) -> UgenState {
    crate::sequencer::new_drum(&self.wavetables, ctl)
  }
}

// XXX move to MIDI manager maybe?

pub fn new_reasonable_of_tables(
  dst: usize,
  wavetables: &Wavetables,
  freq_hz: f32,
  vel: f32,
  ci: usize,
) -> NotegenState {
  NotegenState::new(UgenState::ReasonableSynth(ReasonableSynthState::new(
    dst,
    freq_hz,
    vel,
    wavetables.saw_wavetable.clone(),
    ci,
  )))
}

// XXX move to MIDI manager maybe?

pub fn get_key_state_mut(key_state: &mut Vec<KeyState>, pitch: usize) -> &mut KeyState {
  &mut key_state[pitch - (BOTTOM_NOTE as usize)]
}

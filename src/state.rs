use std::slice::IterMut;
use std::sync::{Arc, Mutex};

use crate::consts::{AUDIO_BUS_LENGTH, BOTTOM_NOTE, NUM_KEYS};
use crate::drum::{DrumControlBlock, DrumSynthState};
use crate::reasonable_synth::{ReasonableControlBlock, ReasonableSynthState};
use crate::ugen::{UgenState, UgensState};
use crate::wavetables::Wavetables;
use crate::webserver::SynthMessage;

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
}

/// Outer vector is list of different busses. Inner vectors each
/// contain one monophonic buffer's worth of audio on
/// each bus.
pub type AudioBusses = Vec<Vec<f32>>;

#[derive(Debug)]
pub struct State {
  pub going: bool,

  // This is NUM_KEYS long, one keystate for every physical key.
  key_state: Vec<KeyState>,

  // audio bus
  pub audio_bus: AudioBusses,

  // This has a varying length as synthesis goes on. Every time we
  // need to allocate a ugen, we try to reuse existing `None`s, but
  // push a new one if necessary.
  pub ugen_state: UgensState,

  // Effects go here
  pub fixed_ugens: UgensState,

  // drum volume
  pub drum_vol: f32,

  // Is the sustain pedal on?
  pub pedal: bool,

  pub write_to_file: bool,

  pub control_blocks: Vec<ControlBlock>,
  pub wavetables: Wavetables,

  pub websocket: Option<tokio::sync::mpsc::Sender<SynthMessage>>,
}

pub type StateGuard = Arc<Mutex<State>>;

impl State {
  pub fn new(buf_size: usize) -> State {
    State {
      going: true,
      key_state: vec![KeyState::Off; NUM_KEYS],
      audio_bus: vec![vec![0.; buf_size]; AUDIO_BUS_LENGTH],
      ugen_state: vec![],
      fixed_ugens: vec![],
      control_blocks: vec![],
      drum_vol: 1.,
      pedal: false,
      write_to_file: true,
      wavetables: Wavetables::new(),
      websocket: None,
    }
  }

  pub fn get_key_state_mut(self: &mut State, pitch: usize) -> &mut KeyState {
    &mut self.key_state[pitch - (BOTTOM_NOTE as usize)]
  }

  pub fn get_key_states(self: &mut State) -> IterMut<'_, KeyState> {
    self.key_state.iter_mut()
  }

  pub fn new_reasonable(&self, freq_hz: f32, vel: f32) -> UgenState {
    UgenState::ReasonableSynth(ReasonableSynthState::new(
      freq_hz,
      vel,
      self.wavetables.saw_wavetable.clone(),
    ))
  }

  pub fn new_drum(&self, freq_hz: f32, freq2_hz: f32) -> UgenState {
    UgenState::DrumSynth(DrumSynthState::new(
      freq_hz,
      freq2_hz,
      self.drum_vol,
      self.wavetables.noise_wavetable.clone(),
    ))
  }
}

use crate::{bass_drum::BassDrumSynthState, reasonable_synth::ReasonableSynthState};

#[derive(Clone)]
pub struct UgenFactory {}

impl UgenFactory {
  pub fn new_reasonable(&self, freq_hz: f32, vel: f32) -> ReasonableSynthState {
    ReasonableSynthState::new(freq_hz, vel)
  }
  pub fn new_drum(&self, freq_hz: f32) -> BassDrumSynthState {
    BassDrumSynthState::new(freq_hz)
  }
  pub fn new() -> Self {
    Self {}
  }
}

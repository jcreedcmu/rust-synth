use rand::Rng;
use std::sync::Arc;

use crate::{
  bass_drum::BassDrumSynthState, reasonable_synth::ReasonableSynthState, synth::TABLE_SIZE,
};

#[derive(Clone)]
pub struct UgenFactory {
  saw_wavetable: Arc<Vec<f32>>,
  noise_wavetable: Arc<Vec<f32>>,
}

impl UgenFactory {
  pub fn new_reasonable(&self, freq_hz: f32, vel: f32) -> ReasonableSynthState {
    ReasonableSynthState::new(freq_hz, vel, self.saw_wavetable.clone())
  }
  pub fn new_drum(&self, freq_hz: f32) -> BassDrumSynthState {
    BassDrumSynthState::new(freq_hz, self.noise_wavetable.clone())
  }
  pub fn new() -> Self {
    // // SINE
    // wavetable[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;

    // // SQUARE
    // wavetable[i] = if (i as f64 / TABLE_SIZE as f64) < 0.5 {
    //   -1.0
    // } else {
    //   1.0
    // };

    // Initialise wavetables
    let mut saw_wavetable = vec![0.0; TABLE_SIZE + 1];
    let mut noise_wavetable = vec![0.0; TABLE_SIZE + 1];

    // Why did we make TABLE_SIZE + 1 with this wraparound? It seems I
    // originally did it so that we can do linear interpolation
    // without worrying about %. Not clear if this really matters for
    // performance.
    for i in 0..TABLE_SIZE {
      saw_wavetable[i] = (2.0 * (i as f64 / TABLE_SIZE as f64) - 1.0) as f32;
    }
    saw_wavetable[TABLE_SIZE] = saw_wavetable[0];

    for i in 0..TABLE_SIZE {
      noise_wavetable[i] = rand::thread_rng().gen_range(-1.0f32..1.0f32);
    }
    noise_wavetable[TABLE_SIZE] = noise_wavetable[0];

    Self {
      saw_wavetable: Arc::new(saw_wavetable),
      noise_wavetable: Arc::new(noise_wavetable),
    }
  }
}

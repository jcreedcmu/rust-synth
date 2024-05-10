use std::sync::Arc;

use rand::Rng;

use crate::synth::TABLE_SIZE;

#[derive(Debug)]
pub struct Wavetables {
  pub saw_wavetable: Arc<Vec<f32>>,
  pub sqr_wavetable: Arc<Vec<f32>>,
  pub noise_wavetable: Arc<Vec<f32>>,
}

impl Wavetables {
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
    let mut sqr_wavetable = vec![0.0; TABLE_SIZE + 1];
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
      sqr_wavetable[i] = if i < TABLE_SIZE / 2 { 1.0 } else { -1.0 };
    }
    sqr_wavetable[TABLE_SIZE] = sqr_wavetable[0];

    for i in 0..TABLE_SIZE {
      noise_wavetable[i] = rand::thread_rng().gen_range(-1.0f32..1.0f32);
    }
    noise_wavetable[TABLE_SIZE] = noise_wavetable[0];

    Self {
      saw_wavetable: Arc::new(saw_wavetable),
      sqr_wavetable: Arc::new(sqr_wavetable),
      noise_wavetable: Arc::new(noise_wavetable),
    }
  }
}

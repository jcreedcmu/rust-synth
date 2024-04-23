use crate::{consts::SAMPLE_RATE_hz, synth::TABLE_SIZE, ugen::Ugen};

#[derive(Clone, Debug)]
pub struct BassDrumSynthState {
  t_s: f32,
  freq_hz: f32,
  phase: f32,
}

impl BassDrumSynthState {
  pub fn new(freq_hz: f32) -> BassDrumSynthState {
    BassDrumSynthState {
      t_s: 0.0,
      phase: 0.0,
      freq_hz,
    }
  }
}

impl Ugen for BassDrumSynthState {
  fn exec(self: &BassDrumSynthState, wavetable: &Vec<f32>) -> f32 {
    let table_phase: f32 = self.phase * ((wavetable.len() - 1) as f32);
    let offset = table_phase.floor() as usize;

    let fpart: f32 = (table_phase as f32) - (offset as f32);

    // linear interp
    let table_val = fpart * wavetable[offset + 1] + (1.0 - fpart) * wavetable[offset];

    0.05 * table_val
  }

  // returns true if should continue note
  fn advance(self: &mut BassDrumSynthState, tick_s: f32) -> bool {
    let BassDrumSynthState {
      freq_hz,
      ref mut phase,
      ref mut t_s,
    } = self;

    *t_s += tick_s;

    const BASS_DRUM_DEBUG_RELEASE_s: f32 = 0.2;
    if *t_s > BASS_DRUM_DEBUG_RELEASE_s {
      false
    } else {
      let bass_drum_freq_hz: f32 = *freq_hz / (TABLE_SIZE as f32);
      *phase += bass_drum_freq_hz / SAMPLE_RATE_hz;
      if *phase > 1. {
        *phase -= 1.;
      }
      true
    }
  }
}

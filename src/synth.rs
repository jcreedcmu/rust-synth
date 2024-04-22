use rand::Rng;

use crate::consts::SAMPLE_RATE_hz;
use crate::state::{BassDrumSynthState, EnvState, ReasonableSynthState, UgenState};

pub const TABLE_SIZE: usize = 4000;

pub struct Synth {
  saw_wavetable: Vec<f32>,
  noise_wavetable: Vec<f32>,
}

impl Synth {
  pub fn new() -> Synth {
    // Initialise wavetable.
    let mut saw_wavetable = vec![0.0; TABLE_SIZE + 1];
    let mut noise_wavetable = vec![0.0; TABLE_SIZE + 1];

    // // SINE
    // wavetable[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;

    // // SQUARE
    // wavetable[i] = if (i as f64 / TABLE_SIZE as f64) < 0.5 {
    //   -1.0
    // } else {
    //   1.0
    // };

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

    Synth {
      saw_wavetable,
      noise_wavetable,
    }
  }

  // FIXME: do the dispatch with traits or something
  fn exec_bass_synth(self: &Synth, state: &mut BassDrumSynthState, samp: &mut f32) {
    let phase: f32 = state.phase;
    let offset = state.phase.floor() as usize;
    // XXX This fpart calculation should be obsolete if I'm fmodding by 1. during update
    let fpart: f32 = (phase as f32) - (offset as f32);
    let table_val =
      fpart * self.noise_wavetable[offset + 1] + (1.0 - fpart) * self.noise_wavetable[offset];

    const bass_drum_freq_hz: f32 = 20.0;
    *samp += 0.01 * table_val;

    let base = bass_drum_freq_hz * (TABLE_SIZE as f32) / SAMPLE_RATE_hz;

    // XXX This phase alteration should be in advance, not in exec, I think. Probably
    // exec should take a non-mutable reference to ugen state.
    state.phase += base;
    if state.phase > 1. {
      state.phase -= 1.;
    }
    wrap_not_mod(&mut state.phase, TABLE_SIZE as f32);
  }

  fn exec_reasonable_synth(self: &Synth, state: &mut ReasonableSynthState, samp: &mut f32) {
    let phase: f32 = state.phase;
    let offset = state.phase.floor() as usize;
    // XXX This fpart calculation should be obsolete if I'm fmodding by 1. during update
    let fpart: f32 = (phase as f32) - (offset as f32);

    // linear interp
    let table_val =
      fpart * self.saw_wavetable[offset + 1] + (1.0 - fpart) * self.saw_wavetable[offset];

    let scale = ugen_env_amp(&state.env_state);
    *samp += (scale as f32) * table_val;
    let base = state.freq_hz * (TABLE_SIZE as f32) / SAMPLE_RATE_hz;

    // XXX This phase alteration should be in advance, not in exec, I think. Probably
    // exec should take a non-mutable reference to ugen state.
    state.phase += base;
    if state.phase > 1. {
      state.phase -= 1.;
    }
    wrap_not_mod(&mut state.phase, TABLE_SIZE as f32);
  }

  fn exec_ugen(self: &Synth, ugen: &mut UgenState, samp: &mut f32) {
    match *ugen {
      UgenState::ReasonableSynth(ref mut state) => self.exec_reasonable_synth(state, samp),
      UgenState::BassDrumSynth(ref mut state) => self.exec_bass_synth(state, samp),
    }
  }

  pub fn exec_maybe_ugen(self: &Synth, ougen: &mut Option<UgenState>, samp: &mut f32) {
    match *ougen {
      None => (),
      Some(ref mut ugen) => self.exec_ugen(ugen, samp),
    }
    advance_ugen(ougen);
  }
}

const ATTACK_s: f32 = 0.005;
const DECAY_s: f32 = 0.005;
const SUSTAIN: f32 = 0.3; // dimensionless
const RELEASE_s: f32 = 0.05;

pub fn ugen_env_amp(env_state: &EnvState) -> f32 {
  match *env_state {
    EnvState::On { t_s, amp, vel } => {
      if t_s < ATTACK_s {
        let a = t_s / ATTACK_s;
        amp * (1.0 - a) + vel * a
      } else if t_s < ATTACK_s + DECAY_s {
        let a = (t_s - ATTACK_s) / DECAY_s;
        vel * (1.0 - a) + vel * SUSTAIN * a
      } else {
        SUSTAIN * vel
      }
    },
    EnvState::Release { t_s, amp } => amp * (1.0 - (t_s / RELEASE_s)),
  }
}

// Advance ugen state forward by 1 audio sample
fn advance_ugen(ugen: &mut Option<UgenState>) {
  let tick_s = 1.0 / SAMPLE_RATE_hz;
  match ugen {
    Some(UgenState::ReasonableSynth(ReasonableSynthState {
      env_state: EnvState::On { ref mut t_s, .. },
      ..
    })) => {
      *t_s += tick_s;
    },
    Some(UgenState::ReasonableSynth(ReasonableSynthState {
      env_state: EnvState::Release { ref mut t_s, .. },
      ..
    })) => {
      *t_s += tick_s;
      if *t_s > RELEASE_s {
        *ugen = None;
      }
    },
    Some(UgenState::BassDrumSynth(BassDrumSynthState { ref mut t_s, .. })) => {
      *t_s += tick_s;
      const BASS_DRUM_DEBUG_RELEASE_s: f32 = 0.2;
      if *t_s > BASS_DRUM_DEBUG_RELEASE_s {
        *ugen = None;
      }
    },
    None => (),
  }
}

fn wrap_not_mod<T: std::cmp::PartialOrd + std::ops::SubAssign + std::convert::From<f32>>(
  x: &mut T,
  size: T,
) {
  if *x >= size {
    *x = 0.0.into();
  }
}

use rand::Rng;

use crate::consts::SAMPLE_RATE_hz;
use crate::state::{BassDrumSynthState, EnvState, ReasonableSynthState, State, UgenState};

pub const TABLE_SIZE: usize = 4000;

pub struct Synth {
  saw_wavetable: Vec<f32>,
  noise_wavetable: Vec<f32>,
  // low pass state
  lowp: Vec<f32>,
  lowp_ix: usize,
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

    let lowp_len = 5;

    Synth {
      saw_wavetable,
      noise_wavetable,
      lowp_len,
      lowp: vec![0.0; lowp_len],
      lowp_ix: 0,
    }
  }

  // FIXME: do the dispatch with traits or something
  fn exec_bass_synth(self: &Synth, state: &BassDrumSynthState, samp: &mut f32) {
    let table_phase: f32 = state.phase * (TABLE_SIZE as f32);
    let offset = table_phase.floor() as usize;

    let fpart: f32 = (table_phase as f32) - (offset as f32);
    let table_val =
      fpart * self.noise_wavetable[offset + 1] + (1.0 - fpart) * self.noise_wavetable[offset];

    *samp += 0.05 * table_val;
  }

  fn exec_reasonable_synth(self: &Synth, state: &ReasonableSynthState, samp: &mut f32) {
    let table_phase: f32 = state.phase * (TABLE_SIZE as f32);
    let offset = table_phase.floor() as usize;

    let fpart: f32 = (table_phase as f32) - (offset as f32);

    // linear interp
    let table_val =
      fpart * self.saw_wavetable[offset + 1] + (1.0 - fpart) * self.saw_wavetable[offset];

    let scale = ugen_env_amp(&state.env_state);
    *samp += (scale as f32) * table_val;
  }

  fn exec_ugen(self: &Synth, ugen: &UgenState, samp: &mut f32) {
    match *ugen {
      UgenState::ReasonableSynth(ref state) => self.exec_reasonable_synth(state, samp),
      UgenState::BassDrumSynth(ref state) => self.exec_bass_synth(state, samp),
    }
  }

  pub fn exec_maybe_ugen(self: &Synth, ougen: &mut Option<UgenState>, samp: &mut f32) {
    match *ougen {
      None => (),
      Some(ref ugen) => {
        self.exec_ugen(ugen, samp);
        // This is where we need &mut ougen. We do in fact want mut
        // access to the *option* because advance may set it to None
        // if the release of some ugen elapses.
        advance_ugen(ougen);
      },
    }
  }

  pub fn synth_frame(self: &mut Synth, s: &mut State) -> f32 {
    let mut samp = 0.0;

    for mut ugen in s.ugen_state.iter_mut() {
      self.exec_maybe_ugen(&mut ugen, &mut samp);
    }
    let Synth {
      ref mut lowp_ix,
      ref mut lowp,
      ..
    } = self;
    let lowp_len = lowp.len();
    *lowp_ix = (*lowp_ix + 1) % lowp_len;
    lowp[*lowp_ix] = samp;
    let out: f32 = { lowp.iter().sum() };
    let len: f32 = lowp_len as f32;

    out / len
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

// Advance ugen state forward by tick_s
// returns true if we should terminate the ugen
fn advance_envelope(env: &mut EnvState, tick_s: f32) -> bool {
  match env {
    EnvState::On { ref mut t_s, .. } => {
      *t_s += tick_s;
      false
    },
    EnvState::Release { ref mut t_s, .. } => {
      *t_s += tick_s;
      *t_s > RELEASE_s
    },
  }
}

// Advance ugen state forward by 1 audio sample
fn advance_ugen(ugen: &mut Option<UgenState>) {
  let tick_s = 1.0 / SAMPLE_RATE_hz;
  match ugen {
    Some(UgenState::ReasonableSynth(ReasonableSynthState {
      freq_hz,
      ref mut phase,
      ref mut env_state,
      ..
    })) => {
      if advance_envelope(env_state, tick_s) {
        *ugen = None;
      } else {
        *phase += *freq_hz / SAMPLE_RATE_hz;
        if *phase > 1. {
          *phase -= 1.;
        }
      }
    },
    Some(UgenState::BassDrumSynth(BassDrumSynthState {
      ref mut t_s,
      ref mut phase,
      freq_hz,
      ..
    })) => {
      *t_s += tick_s;

      const BASS_DRUM_DEBUG_RELEASE_s: f32 = 0.2;
      if *t_s > BASS_DRUM_DEBUG_RELEASE_s {
        *ugen = None;
      } else {
        let bass_drum_freq_hz: f32 = *freq_hz / (TABLE_SIZE as f32);
        *phase += bass_drum_freq_hz / SAMPLE_RATE_hz;
        if *phase > 1. {
          *phase -= 1.;
        }
      }
    },
    None => (),
  }
}

use rand::Rng;

use crate::consts::{ATTACK_s, DECAY_s, RELEASE_s, SAMPLE_RATE_hz, SUSTAIN};
use crate::state::{EnvState, State, UgenState};
use crate::ugen::Ugen;

pub const TABLE_SIZE: usize = 4000;

pub struct Synth {
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

    for i in 0..TABLE_SIZE {
      noise_wavetable[i] = rand::thread_rng().gen_range(-1.0f32..1.0f32);
    }
    noise_wavetable[TABLE_SIZE] = noise_wavetable[0];

    let lowp_len = 5;

    Synth {
      noise_wavetable,
      lowp: vec![0.0; lowp_len],
      lowp_ix: 0,
    }
  }

  fn run_ugen(self: &Synth, ugen: &UgenState) -> f32 {
    match *ugen {
      UgenState::ReasonableSynth(ref u) => u.run(&self.noise_wavetable),
      UgenState::BassDrumSynth(ref u) => u.run(&self.noise_wavetable),
    }
  }

  pub fn exec_maybe_ugen(self: &Synth, ougen: &mut Option<UgenState>, samp: &mut f32) {
    match *ougen {
      None => (),
      Some(ref mut ugen) => {
        *samp += self.run_ugen(ugen);
        if !advance_ugen(ugen) {
          *ougen = None;
        };
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
fn advance_ugen(ugen: &mut UgenState) -> bool {
  let tick_s = 1.0 / SAMPLE_RATE_hz;
  match ugen {
    UgenState::ReasonableSynth(ref mut s) => s.advance(tick_s),
    UgenState::BassDrumSynth(ref mut s) => s.advance(tick_s),
  }
}

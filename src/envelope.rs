use crate::consts::{ATTACK_s, DECAY_s, RELEASE_s, SUSTAIN};

// This is the part of the state that tracks where a note is in its
// ADSR envelope.
#[derive(Clone, Debug)]
pub enum EnvState {
  // Note is activeply sounding. Its pre-existing amplitude at onset
  // time is `amp`. The goal amplitude, at the peak of attack, is
  // `vel`. The amount of time elapsed since its onset is `t_s`.
  On { amp: f32, t_s: f32, vel: f32 },
  // Note is no longer activeply sounding. Its pre-existing amplitude
  // at time of release is `amp`. The amount of time elapsed since its
  // release is `t_s`.
  Release { amp: f32, t_s: f32, dur_s: f32 },
}

impl EnvState {
  // Advance ugen state forward by tick_s
  // returns true if we should keep going, false if we should terminate the ugen
  pub fn advance(&mut self, tick_s: f32) -> bool {
    match self {
      EnvState::On { ref mut t_s, .. } => {
        *t_s += tick_s;
        true
      },
      EnvState::Release {
        ref mut t_s, dur_s, ..
      } => {
        *t_s += tick_s;
        *t_s <= *dur_s
      },
    }
  }

  pub fn amp(&self) -> f32 {
    match self {
      EnvState::On { t_s, amp, vel } => {
        if *t_s < ATTACK_s {
          let a = t_s / ATTACK_s;
          amp * (1.0 - a) + vel * a
        } else if *t_s < ATTACK_s + DECAY_s {
          let a = (t_s - ATTACK_s) / DECAY_s;
          vel * (1.0 - a) + vel * SUSTAIN * a
        } else {
          SUSTAIN * vel
        }
      },
      EnvState::Release { t_s, amp, dur_s } => amp * (1.0 - (t_s / dur_s)),
    }
  }
}

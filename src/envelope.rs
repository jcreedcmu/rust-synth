#[derive(Clone, Debug)]
pub struct Adsr {
  pub attack_s: f32,
  pub decay_s: f32,
  pub sustain: f32,
  pub release_s: f32,
}

// This is the part of the state that tracks where a note is in its
// ADSR envelope.
#[derive(Clone, Debug)]
pub enum EnvPos {
  // Note is activeply sounding. Its pre-existing amplitude at onset
  // time is `amp`. The goal amplitude, at the peak of attack, is
  // `vel`. The amount of time elapsed since its onset is `t_s`.
  // If hold is true, advance will keep us in On. Otherwise, we
  // auto-release once attack and decay phases are over.
  On {
    t_s: f32,
    amp: f32,
    vel: f32,
    hold: bool,
  },
  // Note is no longer activeply sounding. Its pre-existing amplitude
  // at time of release is `amp`. The amount of time elapsed since its
  // release is `t_s`.
  Release {
    t_s: f32,
    amp: f32,
  },
}

#[derive(Clone, Debug)]
pub struct EnvState {
  pub adsr: Adsr,
  pub pos: EnvPos,
}

impl EnvState {
  // Advance ugen state forward by tick_s
  // returns true if we should keep going, false if we should terminate the ugen
  pub fn advance(&mut self, tick_s: f32) -> bool {
    match self.pos {
      EnvPos::On {
        ref mut t_s, hold, ..
      } => {
        *t_s += tick_s;
        if !hold && *t_s > self.adsr.attack_s + self.adsr.decay_s {
          self.pos = EnvPos::Release {
            t_s: 0.,
            amp: self.adsr.sustain,
          };
          return self.adsr.release_s > 0f32;
        }
        true
      },
      EnvPos::Release { ref mut t_s, .. } => {
        *t_s += tick_s;
        *t_s <= self.adsr.release_s
      },
    }
  }

  pub fn amp(&self) -> f32 {
    let Adsr {
      attack_s,
      decay_s,
      sustain,
      release_s,
    } = self.adsr;
    match &self.pos {
      EnvPos::On { t_s, amp, vel, .. } => {
        if *t_s < attack_s {
          let a = t_s / attack_s;
          amp * (1.0 - a) + vel * a
        } else if *t_s < attack_s + decay_s {
          let a = (t_s - attack_s) / decay_s;
          vel * (1.0 - a) + vel * sustain * a
        } else {
          sustain * vel
        }
      },
      EnvPos::Release { t_s, amp } => amp * (1.0 - (t_s / release_s)),
    }
  }
}

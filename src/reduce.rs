use crate::midi::Message;
use crate::state::{EnvState, KeyState, State, UgenState};
use crate::synth::ugen_env_amp;
use crate::util;

fn find_ugen(s: &State, pitch: u8) -> Option<usize> {
  s.ugen_state.iter().position(|x| match x {
    Some(y) => y.pitch == pitch,
    _ => false,
  })
}

fn add_ugen(ns: &mut Vec<Option<UgenState>>, new: UgenState) -> usize {
  let first_free_index = ns.iter().position(|x| match x {
    None => true,
    _ => false,
  });
  match first_free_index {
    None => {
      ns.push(Some(new));
      ns.len() - 1
    }
    Some(i) => {
      ns[i] = Some(new);
      i
    }
  }
}

fn restrike_ugen(ugen: &mut UgenState, vel: f32) {
  ugen.env_state = EnvState::On {
    t_s: 0.0,
    amp: ugen_env_amp(&ugen.env_state),
    vel,
  };
}

fn release_ugen(ugen: &mut Option<UgenState>) {
  match ugen {
    Some(UgenState {
      ref mut env_state, ..
    }) => {
      *env_state = EnvState::Release {
        t_s: 0.0,
        amp: ugen_env_amp(env_state),
      };
    }
    _ => (),
  }
}

// Could have this function return pure data that represents the
// change, then have subsequent function carry it out, so that we hold
// state lock for shorter duration.
pub fn midi_reducer(msg: &Message, s: &mut State) {
  match msg {
    Message::NoteOn {
      pitch,
      channel,
      velocity,
    } => {
      let pitch = *pitch;
      let freq = util::freq_of_pitch(pitch);
      // Is this ugen already being played?
      let pre = find_ugen(&s, pitch);
      let vel = (*velocity as f32) / 1280.0;

      let ugen_ix = match pre {
        Some(i) => {
          match &mut s.ugen_state[i] {
            None => panic!("Invariant Violation: we thought this ugen already existed"),
            Some(ref mut ns) => restrike_ugen(ns, vel),
          };
          i
        }
        None => add_ugen(
          &mut s.ugen_state,
          UgenState {
            phase: 0.0,
            freq_hz: freq,
            pitch,
            env_state: EnvState::On {
              amp: 0.0,
              t_s: 0.0,
              vel,
            },
          },
        ),
      };
      *s.get_key_state_mut(pitch.into()) = KeyState::On { ugen_ix };
    }
    Message::NoteOff { pitch, channel } => {
      let pitch = *pitch;
      let pre = find_ugen(&s, pitch);

      match pre {
        None => println!("warning: NoteOff {} on a ugen already off", pitch),
        Some(ugen_ix) => {
          if s.pedal {
            *s.get_key_state_mut(pitch.into()) = KeyState::Held { ugen_ix };
          } else {
            release_ugen(&mut (s.ugen_state[ugen_ix]));
            *s.get_key_state_mut(pitch.into()) = KeyState::Off;
          }
        }
      }
    }
    Message::PedalOff { .. } => {
      s.pedal = false;
      // Release all pedal-held ugens

      let mut ugen_ixs: Vec<usize> = vec![];

      for ks in s.get_key_states() {
        match ks {
          KeyState::Held { ugen_ix } => {
            ugen_ixs.push(*ugen_ix);
            *ks = KeyState::Off;
          }
          _ => (),
        }
      }
      for ugen_ix in ugen_ixs.iter() {
        release_ugen(&mut (s.ugen_state[*ugen_ix]));
      }
    }
    Message::PedalOn { .. } => {
      s.pedal = true;
    }
  }
}

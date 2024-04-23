use crate::midi::Message;
use crate::state::{KeyState, State};
use crate::ugen::Ugen;
use crate::ugen_factory::UgenFactory;
use crate::util;

pub fn add_ugen_state(s: &mut State, new: impl Ugen + 'static) -> usize {
  add_ugen(&mut s.ugen_state, Box::new(new))
}

// XXX: replay ns with Vec<Option<Box<dyn T>>?
fn add_ugen<T>(ns: &mut Vec<Option<T>>, new: T) -> usize {
  let first_free_index = ns.iter().position(|x| match x {
    None => true,
    _ => false,
  });
  match first_free_index {
    None => {
      ns.push(Some(new));
      ns.len() - 1
    },
    Some(i) => {
      ns[i] = Some(new);
      i
    },
  }
}

// XXX inline?
fn release_ugen(ougen: &mut Option<Box<dyn Ugen>>) {
  match ougen {
    None => (),
    Some(ugen) => ugen.release(),
  }
}

fn ugen_ix_of_key_state(key_state: &KeyState) -> Option<usize> {
  match key_state {
    KeyState::On { ugen_ix } => Some(*ugen_ix),
    KeyState::Held { ugen_ix } => Some(*ugen_ix),
    KeyState::Off => None,
  }
}

// Could have this function return pure data that represents the
// change, then have subsequent function carry it out, so that we hold
// state lock for shorter duration.
pub fn midi_reducer(msg: &Message, fac: &UgenFactory, s: &mut State) {
  match msg {
    Message::NoteOn {
      pitch,
      channel,
      velocity,
    } => {
      let pitch = *pitch;
      let freq = util::freq_of_pitch(pitch);
      // Is this ugen already being played?
      let pre = ugen_ix_of_key_state(s.get_key_state_mut(pitch as usize));
      let vel = (*velocity as f32) / 1280.0;

      let ugen_ix = match pre {
        None => add_ugen(&mut s.ugen_state, Box::new(fac.new_reasonable(freq, vel))),
        Some(ugen_ix) => match &mut s.ugen_state[ugen_ix] {
          None => panic!("Invariant Violation: expected key_state pointed to live ugen"),
          Some(ugen) => {
            ugen.restrike(vel);
            ugen_ix
          },
        },
      };
      *s.get_key_state_mut(pitch.into()) = KeyState::On { ugen_ix };
    },
    Message::NoteOff { pitch, channel } => {
      let pitch = *pitch;
      let pre = ugen_ix_of_key_state(s.get_key_state_mut(pitch as usize));

      match pre {
        None => println!("warning: NoteOff {} on a ugen already off", pitch),
        Some(ugen_ix) => {
          if s.pedal {
            *s.get_key_state_mut(pitch.into()) = KeyState::Held { ugen_ix };
          } else {
            release_ugen(&mut s.ugen_state[ugen_ix]);
            *s.get_key_state_mut(pitch.into()) = KeyState::Off;
          }
        },
      }
    },
    Message::PedalOff { .. } => {
      s.pedal = false;
      // Release all pedal-held ugens

      let mut ugen_ixs: Vec<usize> = vec![];

      for ks in s.get_key_states() {
        match ks {
          KeyState::Held { ugen_ix } => {
            ugen_ixs.push(*ugen_ix);
            *ks = KeyState::Off;
          },
          _ => (),
        }
      }
      for ugen_ix in ugen_ixs.iter() {
        release_ugen(&mut s.ugen_state[*ugen_ix]);
      }
    },
    Message::PedalOn { .. } => {
      s.pedal = true;
    },
  }
}

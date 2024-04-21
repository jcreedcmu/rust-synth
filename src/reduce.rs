use crate::midi::Message;
use crate::state::{EnvState, KeyState, NoteState, State};
use crate::synth::note_env_amp;
use crate::util;

fn find_note(s: &State, pitch: u8) -> Option<usize> {
  s.note_state.iter().position(|x| match x {
    Some(y) => y.pitch == pitch,
    _ => false,
  })
}

fn add_note(ns: &mut Vec<Option<NoteState>>, new: NoteState) -> usize {
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

fn restrike_note(note: &mut NoteState, vel: f32) {
  note.env_state = EnvState::On {
    t_s: 0.0,
    amp: note_env_amp(&note.env_state),
    vel,
  };
}

fn release_note(note: &mut Option<NoteState>) {
  match note {
    Some(NoteState {
      ref mut env_state, ..
    }) => {
      *env_state = EnvState::Release {
        t_s: 0.0,
        amp: note_env_amp(env_state),
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
      // Is this note already being played?
      let pre = find_note(&s, pitch);
      let vel = (*velocity as f32) / 1280.0;

      let note_ix = match pre {
        Some(i) => {
          match &mut s.note_state[i] {
            None => panic!("Invariant Violation: we thought this note already existed"),
            Some(ref mut ns) => restrike_note(ns, vel),
          };
          i
        }
        None => add_note(
          &mut s.note_state,
          NoteState {
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
      s.key_state[pitch as usize] = KeyState::On { note_ix };
    }
    Message::NoteOff { pitch, channel } => {
      let pitch = *pitch;
      let pre = find_note(&s, pitch);

      match pre {
        None => println!("warning: NoteOff {} on a note already off", pitch),
        Some(note_ix) => {
          if s.pedal {
            s.key_state[pitch as usize] = KeyState::Held { note_ix };
          } else {
            release_note(&mut (s.note_state[note_ix]));
            s.key_state[pitch as usize] = KeyState::Off;
          }
        }
      }
    }
    Message::PedalOff { .. } => {
      s.pedal = false;
      // Release all pedal-held notes
      for ks in s.key_state.iter_mut() {
        match ks {
          KeyState::Held { note_ix } => {
            release_note(&mut (s.note_state[*note_ix]));
            *ks = KeyState::Off;
          }
          _ => (),
        }
        //
      }
    }
    Message::PedalOn { .. } => {
      s.pedal = true;
    }
  }
}

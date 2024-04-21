use crate::midi::Message;
use crate::state::{EnvState, NoteState, State};
use crate::synth::note_env_amp;
use crate::util;

fn find_note(s: &State, pitch: u8) -> Option<usize> {
  s.note_state.iter().position(|x| match x {
    Some(y) => y.pitch == pitch,
    _ => false,
  })
}

fn add_note(ns: &mut Vec<Option<NoteState>>, new: NoteState) -> () {
  match ns.iter().position(|x| match x {
    None => true,
    _ => false,
  }) {
    None => ns.push(Some(new)),
    Some(i) => ns[i] = Some(new),
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
      match pre {
        Some(i) => match &mut s.note_state[i] {
          None => panic!("we thought this note already existed"),
          Some(ref mut ns) => restrike_note(ns, vel),
        },
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
      }
    }
    Message::NoteOff { pitch, channel } => {
      let pre = find_note(&s, *pitch);

      match pre {
        None => println!("kinda weird, a noteoff {} on something already off", pitch),
        Some(i) => {
          release_note(&mut (s.note_state[i]));
        }
      }
    }
    _ => (),
  }
}

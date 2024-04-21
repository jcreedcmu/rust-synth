use crate::midi::Message;
use crate::state::{NoteFsm, NoteState, State};
use crate::synth::note_fsm_amp;

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
  note.fsm = NoteFsm::On {
    t_s: 0.0,
    amp: note_fsm_amp(&note.fsm),
    vel,
  };
}

fn release_note(note: &mut Option<NoteState>) {
  match note {
    Some(NoteState { ref mut fsm, .. }) => {
      *fsm = NoteFsm::Release {
        t_s: 0.0,
        amp: note_fsm_amp(fsm),
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
      let freq = 440.0 * 2.0f32.powf(((pitch as f32) - 69.0) / 12.0);
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
            fsm: NoteFsm::On {
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

use anyhow::anyhow;

use crate::midi::Message;
use crate::midi_manager::MidiManagerState;
use crate::notegen::NotegenState;
use crate::state::{
  get_key_state_mut, new_reasonable_of_tables, ComboState, KeyState, State,
  DEFAULT_REASONABLE_CONTROL_BLOCK,
};
use crate::ugen::UgenState;
use crate::util;
use crate::wavetables::Wavetables;
use crate::webserver::SynthMessage;

pub fn add_gen<T>(ns: &mut Vec<Option<T>>, new: T) -> usize {
  let first_free_index = ns.iter().position(|x| match x {
    None => true,
    _ => false,
  });
  let ougen: Option<T> = Some(new);
  match first_free_index {
    None => {
      ns.push(ougen);
      ns.len() - 1
    },
    Some(i) => {
      ns[i] = ougen;
      i
    },
  }
}

fn release_maybe_notegen(onotegen: &mut Option<NotegenState>) {
  match onotegen {
    None => (),
    Some(notegen) => notegen.release(),
  }
}

fn ugen_ix_of_key_state(key_state: &KeyState) -> Option<usize> {
  match key_state {
    KeyState::On { ugen_ix } => Some(*ugen_ix),
    KeyState::Held { ugen_ix } => Some(*ugen_ix),
    KeyState::Off => None,
  }
}

pub fn midi_reducer_inner(
  msg: &Message,
  wavetables: &Wavetables,
  midi_manager: &mut MidiManagerState,
) -> anyhow::Result<()> {
  {
    let MidiManagerState {
      ref dst,
      ref mut pedal,
      ref mut key_state,
      ref mut notegen_state,
      ..
    } = midi_manager;
    match msg {
      Message::NoteOn {
        pitch,
        channel,
        velocity,
      } => {
        let pitch = *pitch;
        let freq = util::freq_of_pitch(pitch);
        // Is this ugen already being played?
        let pre = ugen_ix_of_key_state(get_key_state_mut(key_state, pitch as usize));
        let vel = (*velocity as f32) / 1280.0;

        let ugen_ix = match pre {
          None => {
            let ugen = new_reasonable_of_tables(
              *dst,
              wavetables,
              freq,
              vel,
              DEFAULT_REASONABLE_CONTROL_BLOCK,
            );
            add_gen(notegen_state, ugen)
          },
          Some(ugen_ix) => match &mut notegen_state[ugen_ix] {
            None => panic!("Invariant Violation: expected key_state pointed to live ugen"),
            Some(ugen) => {
              ugen.restrike(vel);
              ugen_ix
            },
          },
        };
        *get_key_state_mut(key_state, pitch.into()) = KeyState::On { ugen_ix };
      },
      Message::NoteOff { pitch, channel } => {
        let pitch = *pitch;
        let pre = ugen_ix_of_key_state(get_key_state_mut(key_state, pitch as usize));

        match pre {
          None => println!("warning: NoteOff {} on a ugen already off", pitch),
          Some(ugen_ix) => {
            if *pedal {
              *get_key_state_mut(key_state, pitch.into()) = KeyState::Held { ugen_ix };
            } else {
              release_maybe_notegen(&mut notegen_state[ugen_ix]);
              *get_key_state_mut(key_state, pitch.into()) = KeyState::Off;
            }
          },
        }
      },
      Message::PedalOff { .. } => {
        *pedal = false;
        // Release all pedal-held ugens

        let mut ugen_ixs: Vec<usize> = vec![];

        for ks in key_state.iter_mut() {
          match ks {
            KeyState::Held { ugen_ix } => {
              ugen_ixs.push(*ugen_ix);
              *ks = KeyState::Off;
            },
            _ => (),
          }
        }
        for ugen_ix in ugen_ixs.iter() {
          release_maybe_notegen(&mut notegen_state[*ugen_ix]);
        }
      },
      Message::PedalOn { .. } => {
        *pedal = true;
      },
    }
    Ok(())
  }
}

// Could have this function return pure data that represents the
// change, then have subsequent function carry it out, so that we hold
// state lock for shorter duration.
pub fn midi_reducer(msg: &Message, state: &mut State) -> anyhow::Result<()> {
  let State {
    gen_state: ComboState { ref websocket, .. },
    fixed_ugens,
    wavetables,
    ..
  } = state;

  if let Some(ws) = websocket {
    ws.try_send(SynthMessage::Midi { msg: msg.clone() })?
  }

  let midi_manager = fixed_ugens
    .iter_mut()
    .find_map(|ugen| match ugen {
      UgenState::MidiManager(m) => Some(m),
      _ => None,
    })
    .map_or_else(|| Err(anyhow!("couldn't find midi manager")), Ok)?;

  midi_reducer_inner(msg, wavetables, midi_manager)
}

use crate::consts::NUM_KEYS;
use crate::notegen::{Notegen, NotegenState};
use crate::state::{AudioBusses, ControlBlocks, KeyState};
use crate::ugen::Ugen;

#[derive(Debug)]
pub struct MidiManagerState {
  dst: usize,
  // Is the sustain pedal on?
  pub pedal: bool,
  // This is NUM_KEYS long, one keystate for every physical key.
  pub key_state: Vec<KeyState>,
  pub notegen_state: Vec<Option<NotegenState>>,
}

impl MidiManagerState {
  pub fn new(dst: usize) -> MidiManagerState {
    MidiManagerState {
      dst,
      pedal: false,
      key_state: vec![KeyState::Off; NUM_KEYS],
      notegen_state: vec![],
    }
  }
}

impl Ugen for MidiManagerState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    for mut onotegen in self.notegen_state.iter_mut() {
      match *onotegen {
        None => (),
        Some(ref mut notegen) => {
          if !notegen.run(bus, tick_s, ctl) {
            *onotegen = None;
          }
        },
      }
    }
    true
  }
}

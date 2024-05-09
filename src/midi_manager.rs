use crate::consts::NUM_KEYS;
use crate::state::{AudioBusses, ControlBlocks, KeyState};
use crate::ugen::{Ugen, UgensState};

#[derive(Debug)]
pub struct MidiManagerState {
  dst: usize,
  // Is the sustain pedal on?
  pub pedal: bool,
  // This is NUM_KEYS long, one keystate for every physical key.
  pub key_state: Vec<KeyState>,
  pub ugen_state: UgensState,
}

impl MidiManagerState {
  pub fn new(dst: usize) -> MidiManagerState {
    MidiManagerState {
      dst,
      pedal: false,
      key_state: vec![KeyState::Off; NUM_KEYS],
      ugen_state: vec![],
    }
  }
}

impl Ugen for MidiManagerState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    for mut ougen in self.ugen_state.iter_mut() {
      match *ougen {
        None => (),
        Some(ref mut ugen) => {
          if !ugen.run(bus, tick_s, ctl) {
            *ougen = None;
          }
        },
      }
    }
    true
  }
  fn release(&mut self) {}
  fn restrike(&mut self, _vel: f32) {}
}

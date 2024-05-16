use crate::consts::NUM_KEYS;
use crate::notegen::NotegenState;
use crate::state::{ControlBlocks, GenState, KeyState};
use crate::ugen::Ugen;

#[derive(Debug)]
pub struct MidiManagerState {
  pub dst: usize,
  // Is the sustain pedal on?
  pub pedal: bool,
  // This is NUM_KEYS long, one keystate for every physical key.
  pub key_state: Vec<KeyState>,
  pub notegen_state: Vec<Option<NotegenState>>,
  pub ci: usize, // control block for new notes
}

impl MidiManagerState {
  pub fn new(dst: usize, ci: usize) -> MidiManagerState {
    MidiManagerState {
      dst,
      pedal: false,
      key_state: vec![KeyState::Off; NUM_KEYS],
      notegen_state: vec![],
      ci,
    }
  }
}

impl Ugen for MidiManagerState {
  fn run(&mut self, mut gen: GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    for mut onotegen in self.notegen_state.iter_mut() {
      if let Some(notegen) = onotegen {
        if !notegen.run(gen.reborrow(), tick_s, ctl) {
          *onotegen = None;
        }
      }
    }
    true
  }
}

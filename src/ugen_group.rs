use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen::{Ugen, UgenState};

#[derive(Debug)]
pub struct UgenGroupState {
  dst: usize,
  pub ugen_state: Vec<Option<UgenState>>,
}

impl UgenGroupState {
  pub fn new(dst: usize) -> UgenGroupState {
    UgenGroupState {
      dst,
      ugen_state: vec![],
    }
  }
}

impl Ugen for UgenGroupState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    for mut ougen in self.ugen_state.iter_mut() {
      if let Some(ugen) = ougen {
        if !ugen.run(bus, tick_s, ctl) {
          *ougen = None;
        }
      }
    }
    true
  }
}

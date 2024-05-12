use crate::reduce::add_gen;
use crate::state::{ControlBlocks, GenState};
use crate::ugen::{Ugen, UgenState};

#[derive(Debug)]
pub struct UgenGroupState {
  dst: usize,
  ugen_state: Vec<Option<UgenState>>,
}

impl UgenGroupState {
  pub fn new(dst: usize) -> UgenGroupState {
    UgenGroupState {
      dst,
      ugen_state: vec![],
    }
  }

  pub fn add(&mut self, ugen: UgenState) {
    add_gen(&mut self.ugen_state, ugen);
  }
}

impl Ugen for UgenGroupState {
  fn run(&mut self, gen: &mut GenState, tick_s: f32, ctl: &ControlBlocks) -> bool {
    for mut ougen in self.ugen_state.iter_mut() {
      if let Some(ugen) = ougen {
        if !ugen.run(gen, tick_s, ctl) {
          *ougen = None;
        }
      }
    }
    true
  }
}

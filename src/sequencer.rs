use crate::drum::drum_adsr;
use crate::reduce::add_ugen_state;
use crate::state::{State, StateGuard};
use crate::util::depoison;
use std::sync::MutexGuard;

// State of the sequencer
#[derive(Debug)]
pub struct Sequencer {
  tab: Vec<Vec<bool>>,
}

pub const SEQ_NUM_INSTRS: usize = 3;
pub const SEQ_PATTERN_LEN: usize = 16;

pub fn sequencer_loop(sg: StateGuard) -> anyhow::Result<()> {
  let mut pos: usize = 0;
  loop {
    {
      let mut s: MutexGuard<State> = depoison(sg.lock())?;
      if !s.going {
        break;
      }

      // if toggle { 660.0 } else { 1760.0 },
      // if toggle { 10.0 } else { 1760.0 },

      if s.sequencer.tab[pos][0] {
        let ugen = s.new_drum(660.0, 10.0, drum_adsr(1.0));
        add_ugen_state(&mut s, ugen);
      }
      if s.sequencer.tab[pos][1] {
        let ugen = s.new_drum(1760.0, 1760.0, drum_adsr(1.0));
        add_ugen_state(&mut s, ugen);
      }
      if s.sequencer.tab[pos][2] {
        let ugen = s.new_drum(6760.0, 5760.0, drum_adsr(0.05));
        add_ugen_state(&mut s, ugen);
      }

      pos = (pos + 1) % SEQ_PATTERN_LEN;
    }
    std::thread::sleep(std::time::Duration::from_millis(125));
  }
  Ok(())
}

impl Sequencer {
  pub fn new() -> Sequencer {
    let mut sequencer: Sequencer = Sequencer {
      tab: vec![vec![false; SEQ_NUM_INSTRS]; SEQ_PATTERN_LEN],
    };
    sequencer
  }

  pub fn set(&mut self, inst: usize, pat: usize, on: bool) {
    self.tab[pat][inst] = on;
  }
}

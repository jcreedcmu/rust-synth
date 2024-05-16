use crate::drum::{drum_adsr, DrumSynthState};
use crate::envelope::Adsr;
use crate::state::{State, StateGuard, DEFAULT_DRUM_CONTROL_BLOCK};
use crate::ugen::UgenState;
use crate::ugen_group::UgenGroupState;
use crate::util::depoison;
use crate::wavetables::Wavetables;
use std::sync::MutexGuard;

// State of the sequencer
#[derive(Debug)]
pub struct Sequencer {
  tab: Vec<Vec<bool>>,
}

pub const SEQ_NUM_INSTRS: usize = 3;
pub const SEQ_PATTERN_LEN: usize = 16;

pub fn new_drum(wavetables: &Wavetables, adsr: Adsr, ctl: usize) -> UgenState {
  UgenState::DrumSynth(DrumSynthState::new(
    adsr,
    wavetables.noise_wavetable.clone(),
    ctl,
  ))
}

fn sequencer_loop_inner(col: &Vec<bool>, wavetables: &Wavetables, group: &mut UgenGroupState) {
  if col[0] {
    group.add(new_drum(
      wavetables,
      drum_adsr(1.0),
      DEFAULT_DRUM_CONTROL_BLOCK + 0,
    ));
  }
  if col[1] {
    group.add(new_drum(
      wavetables,
      drum_adsr(0.5),
      DEFAULT_DRUM_CONTROL_BLOCK + 1,
    ));
  }
  if col[2] {
    group.add(new_drum(
      wavetables,
      drum_adsr(0.05),
      DEFAULT_DRUM_CONTROL_BLOCK + 2,
    ));
  }
}

pub fn sequencer_loop(sg: StateGuard) -> anyhow::Result<()> {
  let mut pos: usize = 0;
  loop {
    {
      let mut s: MutexGuard<State> = depoison(sg.lock())?;
      if !s.going {
        break;
      }

      let State {
        fixed_ugens,
        wavetables,
        sequencer,
        ..
      } = &mut *s;

      let maybe_group = fixed_ugens.iter_mut().find_map(|ugen| match ugen {
        UgenState::UgenGroup(group) => Some(group),
        _ => None,
      });

      if let Some(group) = maybe_group {
        sequencer_loop_inner(&sequencer.tab[pos], wavetables, group);
      } else {
        println!("WARNING: didn't find sequencer ugen group where we expected it");
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

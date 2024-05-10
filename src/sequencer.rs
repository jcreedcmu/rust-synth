use crate::drum::{drum_adsr, DrumSynthState};
use crate::envelope::Adsr;
use crate::reduce::add_gen;
use crate::state::{State, StateGuard};
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

fn new_drum(wavetables: &Wavetables, freq_hz: f32, freq2_hz: f32, adsr: Adsr) -> UgenState {
  UgenState::DrumSynth(DrumSynthState::new(
    freq_hz,
    freq2_hz,
    adsr,
    wavetables.noise_wavetable.clone(),
  ))
}

fn sequencer_loop_inner(col: &Vec<bool>, wavetables: &Wavetables, group: &mut UgenGroupState) {
  if col[0] {
    add_gen(
      &mut group.ugen_state,
      new_drum(wavetables, 660.0, 1.0, drum_adsr(1.0)),
    );
  }
  if col[1] {
    add_gen(
      &mut group.ugen_state,
      new_drum(wavetables, 1760.0, 1000.0, drum_adsr(0.5)),
    );
  }
  if col[2] {
    add_gen(
      &mut group.ugen_state,
      new_drum(wavetables, 6760.0, 5760.0, drum_adsr(0.05)),
    );
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

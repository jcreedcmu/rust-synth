use crate::consts::SAMPLE_RATE;
use crate::state::{NoteFsm, NoteState};

pub const TABLE_SIZE: usize = 4000;

pub struct Synth {
  wavetable: Vec<f32>,
}

impl Synth {
  pub fn new() -> Synth {
    // Initialise wavetable.
    let mut wavetable = vec![0.0; TABLE_SIZE + 1];
    for i in 0..TABLE_SIZE {
      // // SINE
      // wavetable[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;

      // // SQUARE
      // wavetable[i] = if (i as f64 / TABLE_SIZE as f64) < 0.5 {
      //   -1.0
      // } else {
      //   1.0
      // };

      //  SAW
      wavetable[i] = (2.0 * (i as f64 / TABLE_SIZE as f64) - 1.0) as f32;
    }
    wavetable[TABLE_SIZE] = wavetable[0];

    Synth { wavetable }
  }

  pub fn exec_note(self: &Synth, onote: &mut Option<NoteState>, samp: &mut f32) {
    match *onote {
      None => (),
      Some(ref mut note) => {
        let phase: f32 = note.phase;
        let offset = note.phase.floor() as usize;
        let fpart: f32 = (phase as f32) - (offset as f32);

        // linear interp
        let table_val = fpart * self.wavetable[offset + 1] + (1.0 - fpart) * self.wavetable[offset];

        let scale = note_fsm_amp(&note.fsm);
        *samp += (scale as f32) * table_val;
        let base = note.freq * (TABLE_SIZE as f32) / SAMPLE_RATE;
        note.phase += base;
        wrap_not_mod(&mut note.phase, TABLE_SIZE as f32);
      }
    }
    advance_note(onote);
  }
}

const ATTACK: f32 = 0.005; // seconds
const DECAY: f32 = 0.005; // seconds
const SUSTAIN: f32 = 0.3; // dimensionless
const RELEASE: f32 = 0.05; // seconds

pub fn note_fsm_amp(fsm: &NoteFsm) -> f32 {
  match *fsm {
    NoteFsm::On { t_s, amp, vel } => {
      if t_s < ATTACK {
        let a = t_s / ATTACK;
        amp * (1.0 - a) + vel * a
      } else if t_s < ATTACK + DECAY {
        let a = (t_s - ATTACK) / DECAY;
        vel * (1.0 - a) + vel * SUSTAIN * a
      } else {
        SUSTAIN * vel
      }
    }
    NoteFsm::Release { t_s, amp } => amp * (1.0 - (t_s / RELEASE)),
  }
}

fn advance_note(note: &mut Option<NoteState>) {
  match note {
    Some(NoteState {
      fsm: NoteFsm::On { ref mut t_s, .. },
      ..
    }) => {
      *t_s += 1.0 / SAMPLE_RATE;
    }
    Some(NoteState {
      fsm: NoteFsm::Release { ref mut t_s, .. },
      ..
    }) => {
      *t_s += 1.0 / SAMPLE_RATE;
      if *t_s > RELEASE {
        *note = None;
      }
    }
    None => (),
  }
}

fn wrap_not_mod<T: std::cmp::PartialOrd + std::ops::SubAssign + std::convert::From<f32>>(
  x: &mut T,
  size: T,
) {
  if *x >= size {
    *x = 0.0.into();
  }
}
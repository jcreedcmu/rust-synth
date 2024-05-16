use crate::consts::SAMPLE_RATE_hz;
use crate::notegen::NoteMode;
use crate::state::{GenState, State};
use crate::ugen::{Advice, Ugen};

pub const TABLE_SIZE: usize = 512;

pub struct Synth;

impl Synth {
  pub fn new() -> Self {
    Synth
  }

  pub fn synth_buf(self: &mut Synth, s: &mut State) {
    let State {
      audio_bus,
      websocket,
      ..
    } = s;

    // clear the audio busses
    for line in audio_bus.iter_mut() {
      for m in line.iter_mut() {
        *m = 0.;
      }
    }

    let advice = &Advice {
      note_mode: NoteMode::Run,
    };

    for mut ugen in s.fixed_ugens.iter_mut() {
      let gen_state = GenState {
        audio_bus,
        websocket,
        advice,
      };
      // XXX This discards the boolean returned by run
      ugen.run(gen_state, 1.0 / SAMPLE_RATE_hz, &s.control_blocks);
    }
  }
}

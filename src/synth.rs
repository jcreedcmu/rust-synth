use crate::consts::SAMPLE_RATE_hz;
use crate::state::State;
use crate::ugen::Ugen;

pub const TABLE_SIZE: usize = 512;

pub struct Synth;

impl Synth {
  pub fn new() -> Self {
    Synth
  }

  pub fn synth_buf(self: &mut Synth, s: &mut State) {
    // clear the audio busses
    for line in s.audio_bus.iter_mut() {
      for m in line.iter_mut() {
        *m = 0.;
      }
    }

    for mut ugen in s.fixed_ugens.iter_mut() {
      // XXX This discards the boolean returned by run
      ugen.run(&mut s.audio_bus, 1.0 / SAMPLE_RATE_hz, &s.control_blocks);
    }
  }
}

use crate::consts::SAMPLE_RATE_hz;
use crate::state::State;
use crate::ugen::{Ugen, UgenState};

pub const TABLE_SIZE: usize = 4000;

pub struct Synth;

impl Synth {
  pub fn new() -> Self {
    Synth
  }

  pub fn exec_maybe_ugen(self: &Synth, ougen: &mut Option<UgenState>, bus: &mut Vec<f32>) {
    let tick_s = 1.0 / SAMPLE_RATE_hz;
    match *ougen {
      None => (),
      Some(ref mut ugen) => {
        ugen.run(bus);
        if !ugen.advance(tick_s, bus) {
          *ougen = None;
        };
      },
    }
  }

  pub fn synth_frame(self: &mut Synth, s: &mut State) {
    // clear the audio busses
    for m in s.audio_bus.iter_mut() {
      *m = 0.;
    }

    for mut ugen in s.ugen_state.iter_mut() {
      self.exec_maybe_ugen(&mut ugen, &mut s.audio_bus);
    }

    for mut ugen in s.fixed_ugens.iter_mut() {
      self.exec_maybe_ugen(&mut ugen, &mut s.audio_bus);
    }
  }
}

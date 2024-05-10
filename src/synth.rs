use crate::consts::SAMPLE_RATE_hz;
use crate::state::{AudioBusses, ControlBlocks, State};
use crate::ugen::{Ugen, UgenState};

pub const TABLE_SIZE: usize = 512;

pub struct Synth;

impl Synth {
  pub fn new() -> Self {
    Synth
  }

  pub fn exec_maybe_ugen(
    self: &Synth,
    ougen: &mut Option<UgenState>,
    bus: &mut AudioBusses,
    ctl: &ControlBlocks,
  ) {
    let tick_s = 1.0 / SAMPLE_RATE_hz;
    match *ougen {
      None => (),
      Some(ref mut ugen) => {
        if !ugen.run(bus, tick_s, ctl) {
          *ougen = None;
        };
      },
    }
  }

  pub fn synth_buf(self: &mut Synth, s: &mut State) {
    // clear the audio busses
    for line in s.audio_bus.iter_mut() {
      for m in line.iter_mut() {
        *m = 0.;
      }
    }

    for mut ugen in s.fixed_ugens.iter_mut() {
      self.exec_maybe_ugen(&mut ugen, &mut s.audio_bus, &s.control_blocks);
    }
  }
}

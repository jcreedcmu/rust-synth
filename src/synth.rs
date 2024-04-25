use crate::consts::SAMPLE_RATE_hz;
use crate::state::State;
use crate::ugen::{Ugen, UgenState};

pub const TABLE_SIZE: usize = 4000;

pub struct Synth {
  // low pass state
  lowp: Vec<f32>,
  lowp_ix: usize,
}

impl Synth {
  pub fn new() -> Synth {
    let lowp_len = 8;

    Synth {
      lowp: vec![0.0; lowp_len],
      lowp_ix: 0,
    }
  }

  pub fn exec_maybe_ugen(self: &Synth, ougen: &mut Option<UgenState>, samp: &mut f32) {
    let tick_s = 1.0 / SAMPLE_RATE_hz;
    match *ougen {
      None => (),
      Some(ref mut ugen) => {
        *samp += ugen.run();
        if !ugen.advance(tick_s) {
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
      self.exec_maybe_ugen(&mut ugen, &mut s.audio_bus[1]);
    }
    let Synth {
      ref mut lowp_ix,
      ref mut lowp,
      ..
    } = self;

    let lowp_len = lowp.len();
    *lowp_ix = (*lowp_ix + 1) % lowp_len;
    lowp[*lowp_ix] = s.audio_bus[1];
    let out: f32 = lowp.iter().sum::<f32>() / (lowp_len as f32);

    s.audio_bus[0] = out;
  }
}

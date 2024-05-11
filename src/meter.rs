use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen::Ugen;

const METER_AMOUNT: usize = 44100 / 3;

#[derive(Clone, Debug)]
pub struct MeterState {
  src: usize,
  ix: usize,
  memory: Vec<f32>,
}

impl MeterState {
  pub fn new(src: usize) -> Self {
    MeterState {
      src,
      ix: 0,
      memory: vec![0.; METER_AMOUNT],
    }
  }
}

impl Ugen for MeterState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    let len = self.memory.len();
    for bus_ix in 0..bus[0].len() {
      // advance

      let do_tap = |offset: i32, scale: f32| -> f32 {
        scale * self.memory[((self.ix as i32) - offset).rem_euclid(len as i32) as usize]
      };

      let a = 0.99;
      let sig = bus[self.src][bus_ix];
      let mut wet = (1.0 - a) * sig * sig;

      wet += do_tap(1, a);

      self.ix = (self.ix + 1) % len;
      self.memory[self.ix] = wet;
    }
    true
  }
}

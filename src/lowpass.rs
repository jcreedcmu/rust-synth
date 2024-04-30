use crate::state::{AudioBusses, ControlBlocks};
use crate::ugen::Ugen;

const LOW_PASS_AMOUNT: usize = 35000;

#[derive(Clone, Debug)]
pub struct LowpassState {
  src: usize,
  dst: usize,
  ix: usize,
  buffer: Vec<f32>,
}

impl LowpassState {
  pub fn new(src: usize, dst: usize) -> Self {
    LowpassState {
      src,
      dst,
      ix: 0,
      buffer: vec![0.; LOW_PASS_AMOUNT],
    }
  }
}

impl Ugen for LowpassState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    let len = self.buffer.len();
    for bus_ix in 0..bus[0].len() {
      // advance

      let tap = |offset: i32, scale: f32| -> f32 {
        scale * self.buffer[((self.ix as i32) - offset).rem_euclid(len as i32) as usize]
      };

      let wet = 0.03 * bus[self.src][bus_ix] + tap(2, 0.22) + tap(1, 0.75);

      bus[self.dst][bus_ix] = wet;
      self.ix = (self.ix + 1) % len;
      self.buffer[self.ix] = wet;
    }
    true
  }

  fn release(&mut self) {}
  fn restrike(&mut self, vel: f32) {}
}

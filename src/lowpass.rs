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

      let mut sum = bus[self.src][bus_ix]
        + tap(3934, 0.1)
        + tap(5001, 0.5)
        + tap(501, 0.5)
        + tap(102, 0.5)
        + tap(10133, 0.1)
        + tap(12, 0.1)
        + tap(13, 0.1)
        + tap(14, 0.09);

      for i in 0..10 {
        sum += tap(i + 1000, 10.0 / 10.0);
      }
      let wet = sum / 14.1;

      bus[self.dst][bus_ix] = wet;
      self.ix = (self.ix + 1) % len;
      self.buffer[self.ix] = wet;
    }
    true
  }

  fn release(&mut self) {}
  fn restrike(&mut self, vel: f32) {}
}

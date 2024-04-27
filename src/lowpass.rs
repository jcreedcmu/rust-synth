use crate::{state::AudioBusses, ugen::Ugen};

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
      buffer: vec![0.; 16],
    }
  }
}

impl Ugen for LowpassState {
  type ControlBlock = ();

  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32) -> bool {
    let len = self.buffer.len();
    for bus_ix in 0..bus[0].len() {
      bus[self.dst][bus_ix] = self.buffer.iter().sum::<f32>() / (self.buffer.len() as f32);

      // advance
      self.ix = (self.ix + 1) % len;
      self.buffer[self.ix] = bus[self.src][bus_ix];
    }
    true
  }

  fn release(&mut self) {}
  fn restrike(&mut self, vel: f32) {}
}

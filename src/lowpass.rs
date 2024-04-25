use crate::ugen::Ugen;

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
  fn run(&self, bus: &mut Vec<f32>) {
    bus[self.dst] = self.buffer.iter().sum::<f32>() / (self.buffer.len() as f32);
  }

  fn advance(&mut self, tick_s: f32, bus: &Vec<f32>) -> bool {
    let len = self.buffer.len();
    self.ix = (self.ix + 1) % len;
    self.buffer[self.ix] = bus[self.src];
    true
  }

  fn release(&mut self) {}
  fn restrike(&mut self, vel: f32) {}
}

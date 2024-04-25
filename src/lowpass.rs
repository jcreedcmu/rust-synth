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
  fn run(&self) -> f32 {
    // let len = self.buffer.len();
    // *self.ix = (*self.ix + 1) % len;
    // lowp[*lowp_ix] = s.audio_bus[1];
    // let out: f32 = lowp.iter().sum::<f32>() / (len as f32);
    0.
  }

  fn advance(&mut self, tick_s: f32) -> bool {
    false
  }
  fn release(&mut self) {}
  fn restrike(&mut self, vel: f32) {}
}

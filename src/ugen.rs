pub trait Ugen {
  fn exec(&self, wavetable: &Vec<f32>) -> f32;
  fn advance(&mut self, tick_s: f32) -> bool;
}

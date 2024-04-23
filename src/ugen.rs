pub trait Ugen {
  fn run(&self) -> f32;
  fn advance(&mut self, tick_s: f32) -> bool;
}

pub trait Ugen: std::fmt::Debug + Sync + Send {
  fn run(&self) -> f32;
  fn advance(&mut self, tick_s: f32) -> bool;
  fn release(&mut self);
  fn restrike(&mut self, vel: f32);
}

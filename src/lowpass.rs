use crate::audio::BUF_SIZE;
use crate::state::{AudioBusses, ControlBlock, ControlBlocks};
use crate::ugen::{GetSrcBuf, Ugen};

const LOW_PASS_AMOUNT: usize = 35000;

#[derive(Debug)]
pub struct LowpassControlBlock {
  // In the range (0,1).
  // Close to 1: aggressive low-pass, keep previous sample mostly
  // Close to 0: mild low-pass, keep input stream mostly
  pub lowp_param: f32,
}

#[derive(Clone, Debug)]
pub struct LowpassState {
  src: usize,
  dst: usize,
  ix: usize,
  memory: Vec<f32>,
  buf: Vec<f32>,
}

impl LowpassState {
  pub fn new(src: usize, dst: usize) -> Self {
    LowpassState {
      src,
      dst,
      ix: 0,
      memory: vec![0.; LOW_PASS_AMOUNT],
      buf: vec![0.; BUF_SIZE],
    }
  }

  fn ctl_run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &LowpassControlBlock) -> bool {
    let len = self.memory.len();
    for bus_ix in 0..bus[0].len() {
      // advance

      let tap = |offset: i32, scale: f32| -> f32 {
        scale * self.memory[((self.ix as i32) - offset).rem_euclid(len as i32) as usize]
      };

      let lowp_param = ctl.lowp_param;
      let wet = (1. - lowp_param) * bus[self.src][bus_ix] + tap(1, lowp_param);

      bus[self.dst][bus_ix] = wet;
      self.ix = (self.ix + 1) % len;
      self.memory[self.ix] = wet;
    }
    true
  }
}

impl Ugen for LowpassState {
  fn run(&mut self, bus: &mut AudioBusses, tick_s: f32, ctl: &ControlBlocks) -> bool {
    match &ctl[1] {
      // XXX hard coded
      ControlBlock::Low(ctl) => self.ctl_run(bus, tick_s, &ctl),
      _ => false,
    }
  }
  fn release(&mut self) {}
  fn restrike(&mut self, vel: f32) {}
}

impl GetSrcBuf for LowpassState {
  fn get_src_buf(&self) -> &Vec<f32> {
    &self.buf
  }
}

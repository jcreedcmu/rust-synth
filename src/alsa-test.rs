#![allow(unused_variables)]
use alsa::pcm::{Access, Format, HwParams, State, PCM};
use alsa::{Direction, ValueOr};

fn main() {
  // Open default playback device
  let pcm = PCM::new("default", Direction::Playback, false).unwrap();

  // Set hardware parameters: 44100 Hz / Mono / 16 bit
  let hwp = HwParams::any(&pcm).unwrap();
  hwp.set_channels(1).unwrap();
  hwp.set_rate(44100, ValueOr::Nearest).unwrap();
  hwp.set_format(Format::s16()).unwrap();
  hwp.set_access(Access::RWInterleaved).unwrap();
  pcm.hw_params(&hwp).unwrap();
  let io = pcm.io_i16().unwrap();

  // Make sure we don't start the stream too early
  let hwp = pcm.hw_params_current().unwrap();
  let swp = pcm.sw_params_current().unwrap();
  swp
    .set_start_threshold(hwp.get_buffer_size().unwrap())
    .unwrap();
  pcm.sw_params(&swp).unwrap();

  let mut phase: f32 = 0.;

  let mut buf = [0i16; 256];

  loop {
    if false {
      break;
    }
    for (i, a) in buf.iter_mut().enumerate() {
      phase += 440.0 / 44100.0;
      if phase > 1. {
        phase -= 1.;
      }
      *a = ((phase * 2.0 * ::std::f32::consts::PI).sin() * 800.0) as i16;
    }

    assert_eq!(io.writei(&buf[..]).unwrap(), 256);
  }

  // In case the buffer was larger than 2 seconds, start the stream manually.
  if pcm.state() != State::Running {
    pcm.start().unwrap()
  };
  // Wait for the stream to finish playback.
  pcm.drain().unwrap();
}

#![feature(core_intrinsics, try_trait)]
#![allow(unused_imports, unused_variables, unused_mut, dead_code)]

//! Play some sounds.

extern crate portaudio as pad;

mod midi;

use std::error::Error;
use std::f64::consts::PI;
use std::option::NoneError;
use std::thread::sleep;
use std::time::Duration;

const CHANNELS: u32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 150;

type Mostly<T> = Result<T, Box<Error>>;

fn main() {
  match run() {
    Ok(_) => {}
    e => {
      eprintln!("Example failed with the following: {:?}", e);
    }
  }
}

fn wrap<T: std::cmp::PartialOrd + std::ops::SubAssign>(x: &mut T, size: T) {
  if *x >= size {
    *x -= size;
  }
}

// fn check<'a, C>(_cbk: C)
// where
//   C: FnMut(pad::OutputStreamCallbackArgs<'a, f32>) -> pad::StreamCallbackResult,
// {
// }

fn do_other() -> Mostly<()> {
  println!(
    "PortAudio Test: output sine wave. SR = {}, BufSize = {}",
    SAMPLE_RATE, FRAMES_PER_BUFFER
  );

  // Initialise sinusoidal wavetable.
  let mut sine = [0.0; TABLE_SIZE];
  for i in 0..TABLE_SIZE {
    sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
  }
  let mut phase = 0.0; // in frames
  let mut global_t = 0.0; // in seconds
  let pa = pad::PortAudio::new()?;

  let mut settings =
    pa.default_output_stream_settings(CHANNELS as i32, SAMPLE_RATE, FRAMES_PER_BUFFER)?;

  settings.flags = pad::stream_flags::NO_FLAG;

  // This routine will be called by the PortAudio engine when audio is needed. It may called at
  // interrupt level on some machines so don't do anything that could mess up the system like
  // dynamic resource allocation or IO.
  let callback = move |pad::OutputStreamCallbackArgs { buffer, frames, .. }| {
    let mut idx = 0;
    for _ in 0..frames {
      let offset = phase as usize;
      buffer[idx] = sine[offset];
      buffer[idx + 1] = sine[offset];
      phase += if global_t > 0.25 { 1.75 } else { 2.0 };
      wrap(&mut phase, TABLE_SIZE as f64);
      idx += CHANNELS as usize;
      global_t += 1.0 / SAMPLE_RATE;
    }
    if global_t > 0.5 {
      pad::Abort
    } else {
      pad::Continue
    }
  };

  //  check(callback);

  check::<pad::StreamCallbackResult, pad::OutputStreamCallbackArgs<f32>, _>(callback);
  let mut stream = pa.open_non_blocking_stream(settings, callback)?;

  stream.start()?;

  println!("Play for {} seconds.", NUM_SECONDS);

  loop {
    println!("playing...");
    if !stream.is_active()? {
      break;
    }
    pa.sleep(250);
  }

  stream.stop()?;
  stream.close()?;

  println!("Test finished.");
  Ok(())
}

fn check<D, Args, C>(_cbk: C)
where
  C: FnMut(Args) -> D,
{
}

fn run() -> Mostly<()> {
  midi::go(0);
  //  do_other()?;
  sleep(Duration::from_millis(25000));

  Ok(())
}

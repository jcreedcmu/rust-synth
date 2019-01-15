use crate::util::Mostly;
use crate::{Data, State};

use portaudio as pad;

use std::error::Error;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex, MutexGuard};

const CHANNELS: u32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 150;

fn wrap<T: std::cmp::PartialOrd + std::ops::SubAssign>(x: &mut T, size: T) {
  if *x >= size {
    *x -= size;
  }
}

pub struct AudioService {}

impl AudioService {
  pub fn new(data: &Data) -> Mostly<AudioService> {
    println!(
      "PortAudio Test: output sine wave. SR = {}, BufSize = {}",
      SAMPLE_RATE, FRAMES_PER_BUFFER
    );

    // Initialise sinusoidal wavetable.
    let mut sine = [0.0; TABLE_SIZE];
    for i in 0..TABLE_SIZE {
      sine[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;
    }

    let mut global_t = 0.0; // in seconds
    let pa = pad::PortAudio::new()?;

    let mut settings =
      pa.default_output_stream_settings(CHANNELS as i32, SAMPLE_RATE, FRAMES_PER_BUFFER)?;

    settings.flags = pad::stream_flags::NO_FLAG;

    let serv = AudioService {};

    let sg = data.state.clone();

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |pad::OutputStreamCallbackArgs { buffer, frames, .. }| {
      let mut s: MutexGuard<State> = sg.lock().unwrap();
      {
        for ix in 0..frames {
          buffer[2 * ix] = 0.0;
          buffer[2 * ix + 1] = 0.0;
        }
      }
      for note in s.note_state.iter_mut() {
        match note {
          None => (),
          Some(note) => {
            let mut idx = 0;
            for ix in 0..frames {
              let offset = note.phase as usize;
              let samp = (note.amp as f32) * sine[offset];
              buffer[2 * ix] += samp;
              buffer[2 * ix + 1] += samp;
              let base = note.freq * (TABLE_SIZE as f64) / SAMPLE_RATE;
              note.phase += base;
              wrap(&mut note.phase, TABLE_SIZE as f64);
            }
          }
        }
      }
      // phase += if global_t > 0.25 { base * 1.5 } else { base };
      // global_t += 1.0 / SAMPLE_RATE;
      // if global_t > 0.5 {
      //   pad::Abort
      // } else {
      //   pad::Continue
      // }
      pad::Continue
    };

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
    Ok(serv)
  }
}

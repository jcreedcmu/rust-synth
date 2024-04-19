use crate::util::Mostly;
use crate::{Data, NoteFsm, NoteState, State};
use portaudio as pad;
use portaudio::Devices;
use std::sync::{Arc, Mutex, MutexGuard};
pub struct AudioService {}

const CHANNELS: u32 = 2;
const SAMPLE_RATE: f32 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 4000;

impl AudioService {
  pub fn new(data: &Data) -> Mostly<AudioService> {
    let pa = pad::PortAudio::new()?;

    let mut settings =
      pa.default_output_stream_settings(CHANNELS as i32, SAMPLE_RATE as f64, FRAMES_PER_BUFFER)?;

    settings.flags = pad::stream_flags::NO_FLAG;

    let sg = data.state.clone();
    let sg2 = data.state.clone();
    let lowp_len: usize = 30;
    let mut lowp: Vec<f32> = vec![0.0; lowp_len];
    let mut lowp_ix = 0;

    fn wave(x: f32) -> f32 {
      return if x > 0.5 { 1.0 } else { -1.0 };
    }

    let callback = move |pad::OutputStreamCallbackArgs { buffer, frames, .. }| {
      let mut s: MutexGuard<State> = sg.lock().unwrap();
      for ix in 0..frames {
        let mut samp = 0.0;
        s.phase += s.freq / SAMPLE_RATE;
        if s.phase > 1. {
          s.phase -= 1.;
        }
        let out: f32 = 0.01 * wave(s.phase);
        buffer[2 * ix] = out;
        buffer[2 * ix + 1] = out;
      }

      if s.going {
        pad::Continue
      } else {
        pad::Abort
      }
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    loop {
      println!("playing...");
      if !stream.is_active()? {
        break;
      }
      std::thread::sleep(std::time::Duration::from_millis(500));
    }

    stream.stop()?;
    stream.close()?;
    Ok(AudioService {})
  }
}

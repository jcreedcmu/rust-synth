use crate::util::Mostly;
use crate::{Data, NoteFsm, NoteState, State};
use cpal::{
  traits::{DeviceTrait, HostTrait, StreamTrait},
  FromSample, Sample, SizedSample,
};
use std::sync::{Arc, Mutex, MutexGuard};

pub struct AudioService {}

impl AudioService {
  pub fn new(data: &Data) -> Mostly<AudioService> {
    let host = cpal::default_host();
    let device = host
      .default_output_device()
      .expect("failed to find output device");
    println!("[as] Output device: {}", device.name()?);

    let config = device.default_output_config()?;
    println!("[as] Default output config: {:?}", config);

    match config.sample_format() {
      cpal::SampleFormat::F32 => (),
      sample_format => panic!("Unsupported sample format '{sample_format}'"),
    };

    let mut stream_config: cpal::StreamConfig = config.into();
    stream_config.buffer_size = cpal::BufferSize::Fixed(1124);

    println!("stream config {:?}", stream_config);

    let sample_rate = stream_config.sample_rate.0 as f32;
    let channels = stream_config.channels as usize;

    let sg = data.state.clone();
    let sg2 = data.state.clone();

    fn wave(x: f32) -> f32 {
      return if x > 0.5 { 1.0 } else { -1.0 };
    }

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let going = move || -> bool {
      let mut s: MutexGuard<State> = sg2.lock().unwrap();
      s.going
    };

    let stream = device.build_output_stream(
      &stream_config,
      move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut s: MutexGuard<State> = sg.lock().unwrap();

        for frame in data.chunks_mut(channels) {
          for sample in frame.iter_mut() {
            s.phase += s.freq / sample_rate;
            if s.phase > 1. {
              s.phase -= 1.;
            }
            *sample = 0.01 * wave(s.phase);
          }
        }
      },
      err_fn,
      None,
    )?;
    stream.play()?;

    loop {
      if !going() {
        break;
      }
      println!("playing...");
      std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(AudioService {})
  }
}

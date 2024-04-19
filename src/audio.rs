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

    let cc: cpal::StreamConfig = config.into();

    let sample_rate = cc.sample_rate.0 as f32;
    let channels = cc.channels as usize;

    let sg = data.state.clone();

    // Produce a sinusoid of maximum amplitude.
    let mut sample_clock = 0f32;
    let mut next_value = move || {
      let mut s: MutexGuard<State> = sg.lock().unwrap();
      sample_clock = (sample_clock + 1.0) % sample_rate;
      0.001 * (sample_clock * (s.freq as f32) * 2.0 * std::f32::consts::PI / sample_rate).sin()
    };

    let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

    let stream = device.build_output_stream(
      &cc,
      move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        write_data(data, channels, &mut next_value)
      },
      err_fn,
      None,
    )?;
    stream.play()?;

    loop {
      if false {
        break;
      }
      println!("playing...");
      std::thread::sleep(std::time::Duration::from_millis(500));
    }

    Ok(AudioService {})
  }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
  T: Sample + FromSample<f32>,
{
  for frame in output.chunks_mut(channels) {
    let value: T = T::from_sample(next_sample());
    for sample in frame.iter_mut() {
      *sample = value;
    }
  }
}

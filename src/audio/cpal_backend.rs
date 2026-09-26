use super::{Renderer, CHANNELS};
use crate::consts::SAMPLE_RATE_hz;
use crate::util::JoinHandle;
use crate::Args;
use anyhow::anyhow;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

// cpal calls us back whenever the device wants more audio. We ask for
// callbacks of args.buffer_frames frames, but the OS doesn't guarantee that
// (e.g. switching output devices can change it), so the callback renders
// chunks as needed and carries leftover frames over to the next callback.
pub fn start(args: &Args, mut renderer: Renderer) -> anyhow::Result<JoinHandle> {
  let frames = args.buffer_frames;
  let render_thread = std::thread::spawn(move || -> anyhow::Result<()> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or(anyhow!("no default output device"))?;
    println!("Audio output: {}, {frames}-frame buffer", device.name()?);
    let config = StreamConfig {
      channels: CHANNELS as u16,
      sample_rate: SampleRate(SAMPLE_RATE_hz as u32),
      buffer_size: BufferSize::Fixed(frames as u32),
    };

    let done = Arc::new(AtomicBool::new(false));
    let done_cb = done.clone();
    // Index of the next frame of renderer.chunk to send to the device.
    let mut pos = frames;
    let stream = device.build_output_stream(
      &config,
      move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        for frame in data.chunks_mut(CHANNELS) {
          if pos == frames {
            match renderer.render() {
              Ok(true) => {},
              result => {
                if let Err(e) = result {
                  eprintln!("Render error: {e}");
                }
                done_cb.store(true, Ordering::Relaxed);
                renderer.chunk.fill(0.);
              },
            }
            pos = 0;
          }
          frame.fill(renderer.chunk[pos]);
          pos += 1;
        }
      },
      |err| eprintln!("Audio stream error: {err}"),
      None,
    )?;
    stream.play()?;

    // The stream stops when dropped, so hold on to it until the synth quits.
    while !done.load(Ordering::Relaxed) {
      std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
  });
  Ok(render_thread)
}

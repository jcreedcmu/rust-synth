use crate::consts::BUS_OUT;
use crate::synth::Synth;
use crate::util::{depoison, JoinHandle};
use crate::{Args, State, StateGuard};
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::{channel, Sender};
use std::sync::MutexGuard;
use std::time::Instant;

#[cfg(target_os = "linux")]
mod alsa_backend;
#[cfg(target_os = "linux")]
use alsa_backend as backend;

#[cfg(not(target_os = "linux"))]
mod cpal_backend;
#[cfg(not(target_os = "linux"))]
use cpal_backend as backend;

pub struct AudioService {
  pub render_thread: JoinHandle,
}

pub const CHANNELS: usize = 2;

fn vi_to_u8(v: &[i16]) -> &[u8] {
  unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 2) }
}

fn convert_sample(samp_f32: f32) -> i16 {
  (samp_f32 * 32767.0) as i16
}

// Renders the synth one fixed-size chunk (args.buffer_frames long) at a
// time. Backends call render as often as they need to fill whatever buffer
// the device asks for.
struct Renderer {
  args: Args,
  sg: StateGuard,
  synth: Synth,
  file_send: Sender<Vec<i16>>,
  iters: usize,
  // The most recently rendered chunk, mono.
  chunk: Vec<f32>,
}

impl Renderer {
  // Renders the next chunk into self.chunk. Returns false if the synth
  // should stop.
  fn render(&mut self) -> anyhow::Result<bool> {
    let profile = match self.args.profile_interval {
      None => false,
      Some(interval) => self.iters % interval == 0,
    };
    let now = Instant::now();
    {
      let mut s: MutexGuard<State> = depoison(self.sg.lock())?;
      if !s.going {
        return Ok(false);
      }

      self.synth.synth_buf(&mut s);
      self.chunk.copy_from_slice(&s.audio_bus[BUS_OUT]);

      if s.write_to_file {
        let buf = self
          .chunk
          .iter()
          .flat_map(|&x| [convert_sample(x); CHANNELS])
          .collect();
        self.file_send.send(buf)?;
      }
    }
    if profile {
      println!("Elapsed: {:.2?}", now.elapsed());
      println!("Time: {:.2?}", now);
      self.iters = 0;
    }
    self.iters += 1;
    Ok(true)
  }
}

fn start_file_writer() -> anyhow::Result<Sender<Vec<i16>>> {
  let mut file = File::create("/tmp/a.sw")?;
  let (send, recv) = channel::<Vec<i16>>();
  std::thread::spawn(move || -> anyhow::Result<()> {
    let mut n = 0;
    for ref x in recv.iter() {
      n += 1;
      if n % 2000 == 0 {
        n = 0;
        println!("on channel recv'ed {}", x[0]);
      }
      let bytes = vi_to_u8(x);
      file.write_all(bytes)?;
    }
    Ok(())
  });
  Ok(send)
}

impl AudioService {
  pub fn new(args: &Args, state: &StateGuard, synth: Synth) -> anyhow::Result<AudioService> {
    let renderer = Renderer {
      args: args.clone(),
      sg: state.clone(),
      synth,
      file_send: start_file_writer()?,
      iters: 0,
      chunk: vec![0.; args.buffer_frames],
    };
    let render_thread = backend::start(args, renderer)?;
    Ok(AudioService { render_thread })
  }
}

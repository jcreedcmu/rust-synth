use crate::synth::Synth;
use crate::util::Mostly;
use crate::{Data, NoteFsm, NoteState, State};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use std::sync::{Arc, Mutex, MutexGuard};
pub struct AudioService {}

const CHANNELS: u32 = 2;
const FRAMES_PER_BUFFER: u32 = 64;

impl AudioService {
  pub fn new(data: &Data, synth: Synth) -> Mostly<AudioService> {
    let sg = data.state.clone();
    let lowp_len: usize = 5;
    let mut lowp: Vec<f32> = vec![0.0; lowp_len];
    let mut lowp_ix = 0;

    fn wave(x: f32) -> f32 {
      return if x > 0.5 { 1.0 } else { -1.0 };
    }

    // Initialize alsa
    // Open default playback device
    let pcm = PCM::new("hw:2", Direction::Playback, false).unwrap();
    const BUF_SIZE: usize = 64;

    // Set hardware parameters: 44100 Hz / Mono / 16 bit
    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(2).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    hwp.set_buffer_size(BUF_SIZE as i64).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_i16().unwrap();

    // Make sure we don't start the stream too early
    let hwp = pcm.hw_params_current().unwrap();

    let x = hwp.get_buffer_size();
    match x {
      Ok(x) => println!("buffer size {x}"),
      Err(_) => {}
    }

    let swp = pcm.sw_params_current().unwrap();
    swp
      .set_start_threshold(hwp.get_buffer_size().unwrap())
      .unwrap();
    pcm.sw_params(&swp).unwrap();

    let mut phase: f32 = 0.;

    let mut iters: usize = 0;
    let mut buf = [0i16; BUF_SIZE];
    loop {
      {
        let mut s: MutexGuard<State> = sg.lock().unwrap();
        if !s.going {
          break;
        }
        for (i, a) in buf.iter_mut().enumerate() {
          let mut samp = 0.0;

          for mut note in s.note_state.iter_mut() {
            synth.exec_note(&mut note, &mut samp);
          }

          *a = (samp * 32767.0) as i16;
        }
      }
      let _written = io.writei(&buf[..]);
    }

    // Wait for the stream to finish playback.
    pcm.drain().unwrap();

    Ok(AudioService {})
  }
}

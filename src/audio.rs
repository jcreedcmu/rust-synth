use crate::util::Mostly;
use crate::{Data, NoteFsm, NoteState, State};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
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
    // Initialise wavetable.
    let mut wavetable = vec![0.0; TABLE_SIZE + 1];
    for i in 0..TABLE_SIZE {
      // // SINE
      // wavetable[i] = (i as f64 / TABLE_SIZE as f64 * PI * 2.0).sin() as f32;

      // // SQUARE
      // wavetable[i] = if (i as f64 / TABLE_SIZE as f64) < 0.5 {
      //   -1.0
      // } else {
      //   1.0
      // };

      //  SAW
      wavetable[i] = (2.0 * (i as f64 / TABLE_SIZE as f64) - 1.0) as f32;
    }
    wavetable[TABLE_SIZE] = wavetable[0];

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
    hwp.set_buffer_size_min(3).unwrap();
    hwp.set_buffer_size_max(BUF_SIZE as i64).unwrap();
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
            exec_note(&mut note, &wavetable, &mut samp);
          }

          *a = (samp * 16000.0) as i16;
        }
      }
      let _written = io.writei(&buf[..]);
    }

    // In case the buffer was larger than 2 seconds, start the stream manually.
    if pcm.state() != alsa::pcm::State::Running {
      pcm.start().unwrap()
    };

    // Wait for the stream to finish playback.
    pcm.drain().unwrap();

    Ok(AudioService {})
  }
}

const ATTACK: f32 = 0.005; // seconds
const DECAY: f32 = 0.005; // seconds
const SUSTAIN: f32 = 0.3; // dimensionless
const RELEASE: f32 = 0.05; // seconds

pub fn note_fsm_amp(fsm: &NoteFsm) -> f32 {
  match *fsm {
    NoteFsm::On { t, amp, vel } => {
      if t < ATTACK {
        let a = t / ATTACK;
        amp * (1.0 - a) + vel * a
      } else if t < ATTACK + DECAY {
        let a = (t - ATTACK) / DECAY;
        vel * (1.0 - a) + vel * SUSTAIN * a
      } else {
        SUSTAIN * vel
      }
    }
    NoteFsm::Release { t, amp } => amp * (1.0 - (t / RELEASE)),
  }
}

fn advance_note(note: &mut Option<NoteState>) {
  match note {
    Some(NoteState {
      fsm: NoteFsm::On { ref mut t, .. },
      ..
    }) => {
      *t += 1.0 / SAMPLE_RATE;
    }
    Some(NoteState {
      fsm: NoteFsm::Release { ref mut t, .. },
      ..
    }) => {
      *t += 1.0 / SAMPLE_RATE;
      if *t > RELEASE {
        *note = None;
      }
    }
    None => (),
  }
}

fn wrap_not_mod<T: std::cmp::PartialOrd + std::ops::SubAssign + std::convert::From<f32>>(
  x: &mut T,
  size: T,
) {
  if *x >= size {
    *x = 0.0.into();
  }
}

fn exec_note(onote: &mut Option<NoteState>, wavetable: &[f32], samp: &mut f32) {
  match *onote {
    None => (),
    Some(ref mut note) => {
      let phase: f32 = note.phase;
      let offset = note.phase.floor() as usize;
      let fpart: f32 = (phase as f32) - (offset as f32);

      // linear interp
      let table_val = fpart * wavetable[offset + 1] + (1.0 - fpart) * wavetable[offset];

      let scale = note_fsm_amp(&note.fsm);
      *samp += (scale as f32) * table_val;
      let base = note.freq * (TABLE_SIZE as f32) / SAMPLE_RATE;
      note.phase += base;
      wrap_not_mod(&mut note.phase, TABLE_SIZE as f32);
    }
  }
  advance_note(onote);
}

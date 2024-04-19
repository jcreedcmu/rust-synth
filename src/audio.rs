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

    let pa = pad::PortAudio::new()?;

    let mut settings =
      pa.default_output_stream_settings(CHANNELS as i32, SAMPLE_RATE as f64, FRAMES_PER_BUFFER)?;

    settings.flags = pad::stream_flags::NO_FLAG;

    let sg = data.state.clone();
    let sg2 = data.state.clone();
    let lowp_len: usize = 5;
    let mut lowp: Vec<f32> = vec![0.0; lowp_len];
    let mut lowp_ix = 0;

    fn wave(x: f32) -> f32 {
      return if x > 0.5 { 1.0 } else { -1.0 };
    }

    let callback = move |pad::OutputStreamCallbackArgs { buffer, frames, .. }| {
      let mut s: MutexGuard<State> = sg.lock().unwrap();
      for ix in 0..frames {
        let mut samp = 0.0;

        for mut note in s.note_state.iter_mut() {
          exec_note(&mut note, &wavetable, &mut samp);
        }

        buffer[2 * ix] = samp;
        buffer[2 * ix + 1] = samp;
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

const ATTACK: f32 = 0.005; // seconds
const DECAY: f32 = 0.005; // seconds
const SUSTAIN: f32 = 0.5; // dimensionless
const RELEASE: f32 = 0.15; // seconds

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

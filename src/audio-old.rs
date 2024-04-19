use crate::util::Mostly;
use crate::{Data, NoteFsm, NoteState, State};
use crossbeam_channel::unbounded;
use portaudio as pad;
use std::fs::File;
use std::io::Write;

use std::error::Error;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex, MutexGuard};

const CHANNELS: u32 = 2;
const NUM_SECONDS: i32 = 5;
const SAMPLE_RATE: f64 = 44_100.0;
const FRAMES_PER_BUFFER: u32 = 64;
const TABLE_SIZE: usize = 4000;

fn wrap<T: std::cmp::PartialOrd + std::ops::SubAssign>(x: &mut T, size: T) {
  if *x >= size {
    *x -= size;
  }
}

fn wrap_not_mod<T: std::cmp::PartialOrd + std::ops::SubAssign + std::convert::From<f64>>(
  x: &mut T,
  size: T,
) {
  if *x >= size {
    *x = 0.0.into();
  }
}

pub struct AudioService {}

const ATTACK: f64 = 0.005; // seconds
const DECAY: f64 = 0.005; // seconds
const SUSTAIN: f64 = 0.5; // dimensionless
const RELEASE: f64 = 0.15; // seconds

pub fn note_fsm_amp(fsm: &NoteFsm) -> f64 {
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

fn exec_note(onote: &mut Option<NoteState>, wavetable: &[f32], samp: &mut f32) {
  match *onote {
    None => (),
    Some(ref mut note) => {
      let phase: f64 = note.phase;
      let offset = note.phase.floor() as usize;
      let fpart: f32 = (phase as f32) - (offset as f32);

      // linear interp
      let table_val = fpart * wavetable[offset + 1] + (1.0 - fpart) * wavetable[offset];

      let scale = note_fsm_amp(&note.fsm);
      *samp += (scale as f32) * table_val;
      let base = note.freq * (TABLE_SIZE as f64) / SAMPLE_RATE;
      note.phase += base;
      wrap_not_mod(&mut note.phase, TABLE_SIZE as f64);
    }
  }
  advance_note(onote);
}

fn vf_to_u8(v: &[f32]) -> &[u8] {
  unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 4) }
}

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
    let (send, recv) = unbounded::<Vec<f32>>();
    let mut settings =
      pa.default_output_stream_settings(CHANNELS as i32, SAMPLE_RATE, FRAMES_PER_BUFFER)?;

    settings.flags = pad::stream_flags::NO_FLAG;

    // This doesn't seem important anymore, but leaving this here in
    // case I need to stash any state in it
    let serv = AudioService {};

    // state for callback
    let sg = data.state.clone();
    let lowp_len: usize = 30;
    let mut lowp: Vec<f32> = vec![0.0; lowp_len];
    let mut lowp_ix = 0;
    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.

    // Can turn this raw data into a wav file like so:
    // sox -c 2 -r 44100 /tmp/a.f32 /tmp/a.wav
    let mut file = File::create("/tmp/a.f32").unwrap();
    std::thread::spawn(move || {
      let mut n = 0;
      for ref x in recv.iter() {
        n += 1;
        if n % 100 == 0 {
          n = 0;
          println!("on channel recv'ed {}", x[0]);
        }
        let bytes = vf_to_u8(x);
        file.write_all(bytes).unwrap();
      }
    });

    let callback = move |pad::OutputStreamCallbackArgs { buffer, frames, .. }| {
      let mut s: MutexGuard<State> = sg.lock().unwrap();
      for ix in 0..frames {
        let mut samp = 0.0;

        for mut note in s.note_state.iter_mut() {
          exec_note(&mut note, &wavetable, &mut samp);
        }
        lowp_ix = (lowp_ix + 1) % lowp_len;
        lowp[lowp_ix] = samp;
        let out: f32 = { lowp.iter().sum() };
        let len: f32 = lowp.len() as f32;
        buffer[2 * ix] = out / len;
        buffer[2 * ix + 1] = out / len;
      }
      if s.write_to_file {
        send.send(buffer.to_vec()).unwrap();
      }
      pad::Continue
    };

    let mut stream = pa.open_non_blocking_stream(settings, callback)?;

    stream.start()?;

    loop {
      println!("playing...");
      if !stream.is_active()? {
        break;
      }
      pa.sleep(500);
    }

    stream.stop()?;
    stream.close()?;
    Ok(serv)
  }
}

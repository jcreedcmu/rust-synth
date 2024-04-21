use crate::synth::Synth;
use crate::util::Mostly;
use crate::{Data, State};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use dbus::blocking as dbus;
use std::error::Error;
use std::sync::MutexGuard;

pub struct AudioService {}

const CHANNELS: u32 = 2;
const BUF_SIZE: usize = 64;

struct Reservation {
  conn: dbus::Connection,
}

fn dbus_reserve(card: u8) -> Result<Reservation, Box<dyn Error>> {
  // Following the dbus audio device reservation protocol documented in
  // https://git.0pointer.net/reserve.git/tree/reserve.txt
  // and some useful examples are
  // https://github.com/Ardour/ardour/blob/master/libs/ardouralsautil/reserve.c
  // https://gitlab.freedesktop.org/pipewire/pipewire/-/blob/master/src/tools/reserve.c
  let service = &format!("org.freedesktop.ReserveDevice1.Audio{card}");
  let object = &format!("/org/freedesktop/ReserveDevice1/Audio{card}");
  let iface = "org.freedesktop.ReserveDevice1";
  let method = "RequestRelease";
  let priority = 1000; // arbitrary, I'm just hoping it's larger than jack, pulseaudio, pipewire, etc.
  let conn = dbus::Connection::new_session()?;
  let timeout = std::time::Duration::from_millis(5000);

  let proxy = dbus::Proxy::new(service, object, timeout, &conn);
  let (release_result,): (bool,) = proxy.method_call(iface, method, (priority,))?;
  assert!(release_result);

  // "The initial request shall be made with
  // DBUS_NAME_FLAG_DO_NOT_QUEUE and DBUS_NAME_FLAG_ALLOW_REPLACEMENT
  // (exception see below). DBUS_NAME_FLAG_REPLACE_EXISTING shall not
  // be set."
  // (https://git.0pointer.net/reserve.git/tree/reserve.txt)
  let allow_replacement = true;
  let replace_existing = false;
  let do_not_queue = true;
  let reserve_result =
    conn.request_name(service, allow_replacement, replace_existing, do_not_queue)?;
  assert!(reserve_result == dbus::stdintf::org_freedesktop_dbus::RequestNameReply::PrimaryOwner);

  Ok(Reservation { conn })
}

impl AudioService {
  pub fn new(card: u8, data: &Data, synth: Synth) -> Mostly<AudioService> {
    let _reservation = dbus_reserve(card)?;

    let sg = data.state.clone();
    let lowp_len: usize = 5;
    let mut lowp: Vec<f32> = vec![0.0; lowp_len];
    let mut lowp_ix = 0;

    // Initialize alsa
    let device_name = format!("hw:{card}");
    let pcm = PCM::new(&device_name, Direction::Playback, false).unwrap();

    let hwp = HwParams::any(&pcm).unwrap();
    hwp.set_channels(CHANNELS).unwrap();
    hwp.set_rate(44100, ValueOr::Nearest).unwrap();
    hwp.set_format(Format::s16()).unwrap();
    hwp.set_access(Access::RWInterleaved).unwrap();
    hwp.set_buffer_size(BUF_SIZE as i64).unwrap();
    pcm.hw_params(&hwp).unwrap();
    let io = pcm.io_i16().unwrap();

    let hwp = pcm.hw_params_current().unwrap();
    let buffer_size = hwp.get_buffer_size();
    match buffer_size {
      Ok(s) => println!("buffer size is {s}"),
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

        for ch in buf.chunks_mut(CHANNELS as usize) {
          let mut samp = 0.0;

          for mut note in s.note_state.iter_mut() {
            synth.exec_note(&mut note, &mut samp);
          }
          lowp_ix = (lowp_ix + 1) % lowp_len;
          lowp[lowp_ix] = samp;
          let out: f32 = { lowp.iter().sum() };
          let len: f32 = lowp.len() as f32;

          let samp_i16 = (out / len * 32767.0) as i16;

          ch[0] = samp_i16;
          ch[1] = samp_i16;
        }
      }
      let _written = io.writei(&buf[..]);
    }

    // Wait for the stream to finish playback.
    pcm.drain().unwrap();

    Ok(AudioService {})
  }
}

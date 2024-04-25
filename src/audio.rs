use crate::synth::Synth;
use crate::util::Mostly;
use crate::{Args, Data, State};
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use dbus::blocking as dbus;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::mpsc::channel;
use std::sync::MutexGuard;
use std::time::Instant;

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

fn vi_to_u8(v: &[i16]) -> &[u8] {
  unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 2) }
}

impl AudioService {
  pub fn new(args: &Args, data: &Data, mut synth: Synth) -> Mostly<AudioService> {
    let card = args.sound_card;
    let reservation = dbus_reserve(card);
    match reservation {
      Err(e) => println!("Warning: {:?}", e),
      _ => (),
    };

    fn do_profile(args: &Args, iters: usize) -> bool {
      match args.profile_interval {
        None => false,
        Some(interval) => iters % interval == 0,
      }
    }

    let sg = data.state.clone();

    let mut file = File::create("/tmp/a.sw").unwrap();
    let (send, recv) = channel::<Vec<i16>>();
    std::thread::spawn(move || {
      let mut n = 0;
      for ref x in recv.iter() {
        n += 1;
        if n % 2000 == 0 {
          n = 0;
          println!("on channel recv'ed {}", x[0]);
        }
        let bytes = vi_to_u8(x);
        file.write_all(bytes).unwrap();
      }
    });

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
      Err(_) => {},
    }

    let swp = pcm.sw_params_current().unwrap();
    swp
      .set_start_threshold(hwp.get_buffer_size().unwrap())
      .unwrap();
    pcm.sw_params(&swp).unwrap();

    let mut iters: usize = 0;
    let mut buf = [0i16; BUF_SIZE];
    let mut now: Instant = Instant::now();
    loop {
      {
        if do_profile(args, iters) {
          now = Instant::now();
        }
        let mut s: MutexGuard<State> = sg.lock().unwrap();
        if !s.going {
          break;
        }

        for ch in buf.chunks_mut(CHANNELS as usize) {
          let out = synth.synth_frame(&mut s);
          let samp_i16 = (out * 32767.0) as i16;

          ch[0] = samp_i16;
          ch[1] = samp_i16;
        }
        if s.write_to_file {
          send.send(buf.to_vec()).unwrap();
        }
      }
      if do_profile(args, iters) {
        println!("Elapsed: {:.2?}", now.elapsed());
        println!("Time: {:.2?}", now);
        iters = 0;
      }

      iters += 1;

      let _written = io.writei(&buf[..]);
    }

    // Wait for the stream to finish playback.
    pcm.drain().unwrap();

    Ok(AudioService {})
  }
}

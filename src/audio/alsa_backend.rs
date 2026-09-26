use super::{convert_sample, Renderer, CHANNELS};
use crate::util::JoinHandle;
use crate::Args;
use alsa::pcm::{Access, Format, HwParams, PCM};
use alsa::{Direction, ValueOr};
use anyhow::anyhow;
use dbus::blocking as dbus;
use std::error::Error;

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

pub fn start(args: &Args, mut renderer: Renderer) -> anyhow::Result<JoinHandle> {
  let card = args.sound_card.ok_or(anyhow!("--sound-card is required with ALSA"))?;
  let reservation = dbus_reserve(card);
  if let Err(e) = reservation {
    println!("Warning: {:?}", e);
  };

  let frames = args.buffer_frames;
  let render_thread = std::thread::spawn(move || -> anyhow::Result<()> {
    // Initialize alsa
    let device_name = format!("hw:{card}");
    let pcm = PCM::new(&device_name, Direction::Playback, false)?;

    let hwp = HwParams::any(&pcm)?;
    hwp.set_channels(CHANNELS as u32)?;
    hwp.set_rate(44100, ValueOr::Nearest)?;
    hwp.set_format(Format::s16())?;
    hwp.set_access(Access::RWInterleaved)?;
    // Two chunks' worth, so one can be rendered while the other plays.
    hwp.set_buffer_size((frames * 2) as i64)?;
    pcm.hw_params(&hwp)?;
    let io = pcm.io_i16()?;

    let hwp = pcm.hw_params_current()?;
    let buffer_size = hwp.get_buffer_size();
    match buffer_size {
      Ok(s) => println!("buffer size is {s}"),
      Err(_) => {},
    }

    let swp = pcm.sw_params_current()?;
    swp.set_start_threshold(hwp.get_buffer_size()?)?;
    pcm.sw_params(&swp)?;

    let mut buf = vec![0i16; frames * CHANNELS];

    while renderer.render()? {
      for (ch, &samp) in buf.chunks_mut(CHANNELS).zip(renderer.chunk.iter()) {
        ch.fill(convert_sample(samp));
      }
      let _written = io.writei(&buf[..]);
    }

    // Wait for the stream to finish playback.
    pcm.drain()?;
    Ok(())
  });
  Ok(render_thread)
}

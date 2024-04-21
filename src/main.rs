#![allow(unused_variables, unused_mut, dead_code, non_upper_case_globals)]
extern crate midir;

mod audio;
mod consts;
mod midi;
mod reduce;
mod state;
mod synth;
mod util;
use dbus::blocking as dbus;

use midi::Message;
use state::{Data, State};
use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::consts::AUDIO_CARD;

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  // Following the dbus audio device reservation protocol documented in
  // https://git.0pointer.net/reserve.git/tree/reserve.txt
  // and some useful examples are
  // https://github.com/Ardour/ardour/blob/master/libs/ardouralsautil/reserve.c
  // https://gitlab.freedesktop.org/pipewire/pipewire/-/blob/master/src/tools/reserve.c
  let card = consts::AUDIO_CARD;
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

  let state = Arc::new(Mutex::new(State::new()));

  let sg = Data {
    state: state.clone(),
  };

  let sg2 = Data {
    state: state.clone(),
  };

  let _ = std::thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
    // XXX MidiService should just mut-borrow state?
    let ms = midi::MidiService::new(0, move |msg: &Message| {
      let mut s: MutexGuard<State> = sg.state.lock().unwrap();
      reduce::midi_reducer(msg, &mut s);
    });
    let mut input = String::new();
    stdin().read_line(&mut input)?; // wait for next enter key press

    let mut s: MutexGuard<State> = sg2.state.lock().unwrap();
    s.going = false;
    Ok(())
  });

  let ads = audio::AudioService::new(AUDIO_CARD, &Data { state }, synth::Synth::new())?;
  Ok(())
}

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

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let conn = dbus::Connection::new_session()?;
  let proxy = dbus::Proxy::new(
    "org.freedesktop.ReserveDevice1.Audio2",
    "/org/freedesktop/ReserveDevice1/Audio2",
    std::time::Duration::from_millis(5000),
    &conn,
  );
  let (release_result,): (bool,) =
    proxy.method_call("org.freedesktop.ReserveDevice1", "RequestRelease", (1000,))?;
  assert!(release_result);

  let reserve_result =
    conn.request_name("org.freedesktop.ReserveDevice1.Audio2", true, false, false)?;
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

  let ads = audio::AudioService::new(&Data { state }, synth::Synth::new())?;
  Ok(())
}

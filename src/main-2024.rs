#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
extern crate midir;

mod audio;
mod beep;
mod util;

use midir::{Ignore, MidiIO, MidiInput, MidiInputPort, MidiOutput};
use std::error::Error;
use std::io::{stdin, stdout, Write};

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn do_midi_stuff() -> Result<(), Box<dyn Error>> {
  let mut midi_in = MidiInput::new("midir input")?;
  midi_in.ignore(Ignore::None);

  let midi_device_num = 1;
  let in_port = midi_in
    .ports()
    .get(midi_device_num)
    .ok_or("Invalid port number")?
    .clone();

  println!("\nOpening connections");
  let in_port_name = midi_in.port_name(&in_port)?;

  // _conn_in needs to be a named binding, because it needs to be kept alive until the end of the scope
  let _conn_in = midi_in.connect(
    &in_port,
    "midir-print",
    move |stamp, message, _| {
      println!("{}: {:?} (len = {})", stamp, message, message.len());
    },
    (),
  )?;

  let mut input = String::new();
  stdin().read_line(&mut input)?; // wait for next enter key press
  Ok(())
}

fn run() -> Result<(), Box<dyn Error>> {
  let _ = std::thread::spawn(move || {
    let _ = do_midi_stuff();
  });

  let ads = audio::AudioService::new()?;
  Ok(())
}

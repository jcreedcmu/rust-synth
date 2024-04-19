#![allow(unused_imports, unused_variables, unused_mut, dead_code)]
extern crate midir;

use std::error::Error;
use std::io::{stdin, stdout, Write};

use midir::{Ignore, MidiIO, MidiInput, MidiOutput};

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  let mut midi_in = MidiInput::new("midir  input")?;
  midi_in.ignore(Ignore::None);

  let in_port = select_port(&midi_in, "input")?;

  println!("\nOpening connections");
  let in_port_name = midi_in.port_name(&in_port)?;

  // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
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

fn select_port<T: MidiIO>(midi_io: &T, descr: &str) -> Result<T::Port, Box<dyn Error>> {
  println!("Available {} ports:", descr);
  let midi_ports = midi_io.ports();
  for (i, p) in midi_ports.iter().enumerate() {
    println!("{}: {}", i, midi_io.port_name(p)?);
  }
  print!("Please select {} port: ", descr);
  stdout().flush()?;
  let mut input = String::new();
  stdin().read_line(&mut input)?;
  let port = midi_ports
    .get(input.trim().parse::<usize>()?)
    .ok_or("Invalid port number")?;
  Ok(port.clone())
}

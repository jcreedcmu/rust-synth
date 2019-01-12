extern crate coremidi;
use coremidi::{Client, PacketList, Source, Sources};
use std::env;
use std::error::Error;
use std::fmt;
use std::thread::sleep;
use std::time::Duration;

// pub fn input_port<F>(&self, name: &str, callback: F) -> Result<InputPort, OSStatus>
//         where F: FnMut(&PacketList) + Send + 'static {

#[derive(Debug)]
pub enum MidiError {
  Os(i32),
  BadSource(usize),
}

impl From<i32> for MidiError {
  fn from(x: i32) -> MidiError {
    MidiError::Os(x)
  }
}

impl fmt::Display for MidiError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MidiError::Os(x) => write!(f, "Os error {}", x),
      MidiError::BadSource(x) => write!(f, "Bad source {}", x),
    }
  }
}

impl Error for MidiError {
  fn description(&self) -> &str {
    "invalid first item to double"
  }

  fn cause(&self) -> Option<&Error> {
    // Generic error, underlying cause isn't tracked.
    None
  }
}

pub fn go(source_index: usize) -> Result<(), MidiError> {
  //  println!("Source index: {}", source_index);

  let source = match Source::from_index(source_index) {
    Some(x) => x,
    None => Err(MidiError::BadSource(source_index))?,
  };

  let client = Client::new("example-client")?;
  println!("hi");
  let input_port = client
    .input_port("example-port", |packet_list: &PacketList| {
      println!("{}", packet_list);
    })
    .unwrap();
  input_port.connect_source(&source)?;
  sleep(Duration::from_millis(25000));

  //  input_port.disconnect_source(&source)?;
  Ok(())
}

fn get_source_index() -> usize {
  let mut args_iter = env::args();
  let tool_name = args_iter
    .next()
    .and_then(|path| {
      path
        .split(std::path::MAIN_SEPARATOR)
        .last()
        .map(|v| v.to_string())
    })
    .unwrap_or("receive".to_string());

  match args_iter.next() {
    Some(arg) => match arg.parse::<usize>() {
      Ok(index) => {
        if index >= Sources::count() {
          println!("Source index out of range: {}", index);
          std::process::exit(-1);
        }
        index
      }
      Err(_) => {
        println!("Wrong source index: {}", arg);
        std::process::exit(-1);
      }
    },
    None => {
      println!("Usage: {} <source-index>", tool_name);
      println!("");
      println!("Available Sources:");
      print_sources();
      std::process::exit(-1);
    }
  }
}

fn print_sources() {
  for (i, source) in Sources.into_iter().enumerate() {
    match source.display_name() {
      Some(display_name) => println!("[{}] {}", i, display_name),
      None => (),
    }
  }
}

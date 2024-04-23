#![allow(unused_variables, unused_mut, dead_code, non_upper_case_globals)]

mod audio;
mod bass_drum;
mod consts;
mod midi;
mod reasonable_synth;
mod reduce;
mod state;
mod synth;
mod ugen;
mod util;
mod wavetables;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use clap::Parser;
use midi::{Message, MidiService};
use reduce::add_ugen_state;
use serde::Deserialize;
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

#[derive(Deserialize, Debug)]
struct WebMessage {
  message: usize,
}

fn do_thing(s: &mut State) {
  let ugen = s.new_drum(1000.0);
  add_ugen_state(s, ugen);
}

#[post("/api/action")]
async fn action(
  message: web::Json<WebMessage>,
  extra: actix_web::web::Data<Arc<Mutex<State>>>,
) -> impl Responder {
  let mut s = extra.lock().unwrap();
  do_thing(&mut s);
  println!("got: {:?}", message);
  HttpResponse::Ok().body("{}")
}

#[actix_web::main]
async fn web_serve(sg: Arc<Mutex<State>>) -> std::io::Result<()> {
  HttpServer::new(move || {
    App::new()
      .app_data(actix_web::web::Data::new(sg.clone()))
      .service(action)
      .service(actix_files::Files::new("/", "./public").index_file("index.html"))
  })
  .bind(("127.0.0.1", 8000))?
  .run()
  .await
}

fn mk_web_thread(sg: Arc<Mutex<State>>) {
  std::thread::spawn(move || {
    web_serve(sg).unwrap();
  });
}

fn mk_sequencer_thread(sg: Arc<Mutex<State>>) {
  std::thread::spawn(move || {
    let mut toggle: bool = true;
    loop {
      std::thread::sleep(std::time::Duration::from_millis(500));
      {
        let mut s: MutexGuard<State> = sg.lock().unwrap();
        if !s.going {
          break;
        }
        let ugen = s.new_drum(if toggle { 660.0 } else { 1760.0 });
        add_ugen_state(&mut s, ugen);
        toggle = !toggle;
      }
    }
  });
}

fn mk_midi_service(sg: Arc<Mutex<State>>) -> Result<MidiService, Box<dyn Error>> {
  midi::MidiService::new(0, move |msg: &Message| {
    let mut s: MutexGuard<State> = sg.lock().unwrap();
    reduce::midi_reducer(msg, &mut s);
  })
}

fn mk_stdin_thread(sg: Arc<Mutex<State>>) {
  std::thread::spawn(move || -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
      let mut input = String::new();
      stdin().read_line(&mut input)?; // wait for next enter key press

      match input.as_str() {
        "\n" => {
          let mut s: MutexGuard<State> = sg.lock().unwrap();
          s.going = false;
          break;
        },
        "k\n" => {
          let mut s: MutexGuard<State> = sg.lock().unwrap();
          let ugen = s.new_drum(440.0);
          add_ugen_state(&mut s, ugen);
        },
        _ => println!("Didn't recognize {input}."),
      }
    }
    Ok(())
  });
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
  // Sound card
  #[arg(short = 'c', long, env)]
  sound_card: u8,

  // Profiling interval, measured in number of BUF_SIZE-long audio sample generation periods
  #[arg(long, env)]
  profile_interval: Option<usize>,
}

fn run() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();
  let state = Arc::new(Mutex::new(State::new()));

  let ms = mk_midi_service(state.clone())?;
  mk_sequencer_thread(state.clone());
  mk_stdin_thread(state.clone());
  mk_web_thread(state.clone());

  let ads = audio::AudioService::new(&args, &Data { state }, synth::Synth::new())?;
  Ok(())
}

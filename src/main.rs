#![allow(unused_variables, unused_mut, dead_code, non_upper_case_globals)]

mod audio;
mod consts;
mod drum;
mod envelope;
mod lowpass;
mod midi;
mod reasonable_synth;
mod reduce;
mod sequencer;
mod state;
mod synth;
mod ugen;
mod util;
mod wavetables;
mod webserver;

use audio::{BUF_SIZE, CHANNELS};
use clap::Parser;
use consts::{BUS_DRY, BUS_OUT};
use drum::DrumControlBlock;
use lowpass::LowpassState;
use midi::{Message, MidiService};
use reduce::{add_fixed_ugen_state, add_ugen_state};
use sequencer::sequencer_loop;
use state::{State, StateGuard};
use util::{depoison, JoinHandle};
use webserver::{WebAction, WebMessage, WebOrSubMessage};

use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex, MutexGuard};

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn reduce_web_message(m: &WebMessage, s: &mut State) {
  match m.message {
    WebAction::Drum => {
      let ugen = s.new_drum(1000.0, 2000.0, 1.0);
      add_ugen_state(s, ugen);
    },
    WebAction::Quit => {
      s.going = false;
    },
    WebAction::SetVolume { vol } => match &mut s.control_blocks[0] {
      state::ControlBlock::Drum(DrumControlBlock { vol: ctl_vol }) => {
        *ctl_vol = (vol as f32) / 100.0;
      },
      _ => {
        println!("Unexpected control block");
      },
    },
    WebAction::SetSequencer { inst, pat, on } => {
      s.sequencer.set(inst, pat, on);
    },
  }
}

fn reduce_web_or_sub_message(m: &WebOrSubMessage, s: &mut State) {
  match m {
    WebOrSubMessage::WebMessage(m) => {
      reduce_web_message(m, s);
    },
    WebOrSubMessage::SubMessage(tx) => {
      s.websocket = Some(tx.clone());
    },
  }
}

fn mk_web_thread(sg: StateGuard) -> (JoinHandle, JoinHandle) {
  webserver::start(move |msg| {
    let mut s = depoison(sg.lock())?;
    reduce_web_or_sub_message(&msg, &mut s);
    Ok(())
  })
}

fn mk_sequencer_thread(sg: StateGuard) -> JoinHandle {
  std::thread::spawn(move || -> anyhow::Result<()> {
    sequencer_loop(sg)?;
    Ok(())
  })
}

fn mk_midi_service(sg: StateGuard) -> anyhow::Result<MidiService> {
  midi::MidiService::new(0, move |msg: &Message| -> anyhow::Result<()> {
    let mut s: MutexGuard<State> = depoison(sg.lock())?;
    reduce::midi_reducer(msg, &mut s)?;
    Ok(())
  })
}

fn mk_stdin_thread(sg: StateGuard) -> JoinHandle {
  std::thread::spawn(move || -> anyhow::Result<()> {
    loop {
      let mut input = String::new();
      stdin().read_line(&mut input)?; // wait for next enter key press

      match input.as_str() {
        "\n" => {
          let mut s: MutexGuard<State> = depoison(sg.lock())?;
          s.going = false;
          break;
        },
        "k\n" => {
          let mut s: MutexGuard<State> = depoison(sg.lock())?;
          let ugen = s.new_drum(440.0, 440.0, 1.0);
          add_ugen_state(&mut s, ugen);
        },
        _ => println!("Didn't recognize {input}."),
      }
    }
    Ok(())
  })
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

  let mono_buf_size = BUF_SIZE / (CHANNELS as usize);
  let mut state = State::new(mono_buf_size);

  add_fixed_ugen_state(
    &mut state,
    ugen::UgenState::Lowpass(LowpassState::new(BUS_DRY, BUS_OUT)),
  );

  let state = Arc::new(Mutex::new(state));

  let ms = mk_midi_service(state.clone())?;
  mk_sequencer_thread(state.clone());
  mk_stdin_thread(state.clone());
  mk_web_thread(state.clone());

  let ads = audio::AudioService::new(&args, &state, synth::Synth::new())?;
  Ok(())
}

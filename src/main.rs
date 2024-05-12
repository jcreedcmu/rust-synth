#![allow(unused_variables, unused_mut, dead_code, non_upper_case_globals)]

mod audio;
mod consts;
mod drum;
mod envelope;
mod gain;
mod lowpass;
mod meter;
mod midi;
mod midi_manager;
mod notegen;
mod reasonable_synth;
mod reduce;
mod sequencer;
mod state;
mod synth;
mod ugen;
mod ugen_group;
mod util;
mod wavetables;
mod webserver;

use audio::{BUF_SIZE, CHANNELS};
use clap::Parser;
use consts::{BUS_DRY, BUS_OUT};
use lowpass::LowpassState;
use midi::{Message, MidiService};
use midi_manager::MidiManagerState;
use sequencer::sequencer_loop;
use state::{State, StateGuard};
use ugen::UgenState;
use ugen_group::UgenGroupState;
use util::{depoison, JoinHandle, UnitHandle};
use webserver::{WebAction, WebMessage, WebOrSubMessage};

use std::error::Error;
use std::io::stdin;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::drum::drum_adsr;

fn main() {
  match run() {
    Ok(_) => (),
    Err(err) => println!("Error: {}", err),
  }
}

fn reduce_web_message(m: WebMessage, s: &mut State) {
  match m.message {
    WebAction::Drum => {
      let ugen = s.new_drum(440.0, 440.0, drum_adsr(1.0));
      add_ugen_to_group(&mut s.fixed_ugens, ugen);
    },
    WebAction::Quit => {
      s.going = false;
    },
    WebAction::SetSequencer { inst, pat, on } => {
      s.sequencer.set(inst, pat, on);
    },
    WebAction::Reconfigure { specs } => {
      s.fixed_ugens = specs //
        .into_iter()
        .map(UgenState::new)
        .collect();
    },
    WebAction::SetControlBlock { index, ctl } => {
      // XXX what if we need to extend the vector?
      s.control_blocks[index] = ctl;
    },
  }
}

fn reduce_web_or_sub_message(m: WebOrSubMessage, s: &mut State) {
  match m {
    WebOrSubMessage::WebMessage(m) => {
      reduce_web_message(m, s);
    },
    WebOrSubMessage::SubMessage(tx) => {
      s.gen_state.websocket = Some(tx.clone());
    },
  }
}

fn mk_web_thread(sg: StateGuard) -> (UnitHandle, UnitHandle) {
  webserver::start(move |msg| {
    let mut s = depoison(sg.lock())?;
    reduce_web_or_sub_message(msg, &mut s);
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

fn add_ugen_to_group(ugens: &mut Vec<UgenState>, ugen: UgenState) {
  let maybe_group = ugens.iter_mut().find_map(|ugen| match ugen {
    UgenState::UgenGroup(group) => Some(group),
    _ => None,
  });
  if let Some(group) = maybe_group {
    group.add(ugen);
  } else {
    println!("couldn't find ugen group");
  }
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
          let ugen = s.new_drum(440.0, 440.0, drum_adsr(1.0));
          add_ugen_to_group(&mut s.fixed_ugens, ugen);
        },
        _ => println!("Didn't recognize {input}."),
      }
    }
    Ok(())
  })
}

#[derive(Parser, Debug, Clone)]
#[command(version, about)]
pub struct Args {
  // Sound card
  #[arg(short = 'c', long, env)]
  sound_card: u8,

  // Profiling interval, measured in number of BUF_SIZE-long audio sample generation periods
  #[arg(long, env)]
  profile_interval: Option<usize>,
}

fn setup_ctrlc_handler(sg: StateGuard) {
  ctrlc::set_handler(move || {
    let mut s: MutexGuard<State> = sg.lock().unwrap();
    reduce_web_message(
      WebMessage {
        message: WebAction::Quit,
      },
      &mut s,
    );
  })
  .expect("Error setting Ctrl-C handler");
}

fn run() -> Result<(), Box<dyn Error>> {
  let args = Args::parse();

  let mono_buf_size = BUF_SIZE / (CHANNELS as usize);
  let mut state = State::new(mono_buf_size);

  state.fixed_ugens = vec![
    ugen::UgenState::MidiManager(MidiManagerState::new(BUS_DRY)),
    ugen::UgenState::UgenGroup(UgenGroupState::new(BUS_DRY)),
    ugen::UgenState::Lowpass(LowpassState::new(BUS_DRY, BUS_OUT)),
  ];

  let state = Arc::new(Mutex::new(state));

  let ms = mk_midi_service(state.clone())?;
  mk_sequencer_thread(state.clone());
  mk_stdin_thread(state.clone());
  mk_web_thread(state.clone());
  setup_ctrlc_handler(state.clone());

  let ads = audio::AudioService::new(&args, &state, synth::Synth::new())?;
  ads.render_thread.join().unwrap()?;
  Ok(())
}

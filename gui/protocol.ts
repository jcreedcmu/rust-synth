export type UgenSpec =
  | { t: 'lowPass', src: number, dst: number }
  | { t: 'midiManager', dst: number }
  | { t: 'ugenGroup', dst: number }
  | { t: 'meter', src: number }
  ;

export type Tap = {
  pos: number, // integer >0
  weight: number, // float in [0,1], together with selfWeight maybe should add up to < 1
}

export type LowpassControlBlock = {
  selfWeight: number,
  taps: Tap[],
}

export type ControlBlock =
  | { t: 'Reasonable' }
  | { t: 'Drum', vol: number }
  | { t: 'Low' } & LowpassControlBlock
  | { t: 'Gain', scale: number }
  ;

export type WebAction =
  | { t: 'quit' }
  | { t: 'drum' }
  | { t: 'setVolume', vol: number }
  | { t: 'setLowpassParam', lowp_param: number } // XXX deprecated
  | { t: 'setLowpassConfig', lowp_cfg: LowpassControlBlock } // XXX deprecated
  | { t: 'setControlBlock', index: number, ctl: ControlBlock }
  | { t: 'setSequencer', inst: number, pat: number, on: boolean }
  | { t: 'reconfigure', specs: UgenSpec[] }
  ;

export type WebMessage = {
  message: WebAction
};

export type MidiMessage =
  | { t: 'noteOn', pitch: number, channel: number, velocity: number }
  | { t: 'noteOff', pitch: number, channel: number }
  | { t: 'pedalOn' }
  | { t: 'pedalOff' }
  ;

export type SynthMessage =
  | { t: 'midi', msg: MidiMessage }
  | { t: 'meter', level: number } // rms
  ;

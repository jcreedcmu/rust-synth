export type UgenSpec =
  | { t: 'lowPass', src: number, dst: number }
  | { t: 'allPass', src: number, dst: number, ctl: number }
  | { t: 'midiManager', dst: number }
  | { t: 'ugenGroup', dst: number }
  | { t: 'meter', src: number }
  | { t: 'gain', src: number, dst: number }
  ;

export type TapType =
  | { t: 'Rec' }
  | { t: 'Input' }
  ;

export type Tap = {
  tp: TapType,
  pos: number, // integer >0
  weight: number, // float in [0,1], together with selfWeight maybe should add up to < 1
}

export type LowpassControlBlock = {
  taps: Tap[],
}

export type AllpassControlBlock = {
  gain: number,
  delay: number,
}

export type ControlBlock =
  | { t: 'Reasonable' }
  | { t: 'Drum', vol: number }
  | { t: 'Low' } & LowpassControlBlock
  | { t: 'All' } & AllpassControlBlock
  | { t: 'Gain', scale: number }
  ;

export type WebMessage =
  | { t: 'quit' }
  | { t: 'drum' }
  | { t: 'setControlBlock', index: number, ctl: ControlBlock }
  | { t: 'setSequencer', inst: number, pat: number, on: boolean }
  | { t: 'reconfigure', specs: UgenSpec[] }
  ;

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

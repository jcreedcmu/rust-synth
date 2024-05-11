export type UgenSpec =
  | { t: 'lowPass', src: number, dst: number }
  | { t: 'midiManager', dst: number }
  | { t: 'ugenGroup', dst: number }
  ;

export type Tap = {
  pos: number, // integer >0
  weight: number, // float in [0,1], together with selfWeight maybe should add up to < 1
}

export type LowpassControlBlock = {
  selfWeight: number,
  taps: Tap[],
}

export type WebAction =
  | { t: 'quit' }
  | { t: 'drum' }
  | { t: 'setVolume', vol: number }
  | { t: 'setLowpassParam', lowp_param: number }
  | { t: 'setLowpassConfig', lowp_cfg: LowpassControlBlock }
  | { t: 'setSequencer', inst: number, pat: number, on: boolean }
  | { t: 'reconfigure', specs: UgenSpec[] }
  ;

export type WebMessage = {
  message: WebAction
};

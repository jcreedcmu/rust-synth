export type UgenSpec =
  | { t: 'lowPass', src: number, dst: number }
  | { t: 'midiManager', dst: number }
  | { t: 'ugenGroup', dst: number }
  ;

export type WebAction =
  | { t: 'quit' }
  | { t: 'drum' }
  | { t: 'setVolume', vol: number }
  | { t: 'setLowpassParam', lowp_param: number }
  | { t: 'setSequencer', inst: number, pat: number, on: boolean }
  | { t: 'reconfigure', specs: UgenSpec[] }
  ;

export type WebMessage = {
  message: WebAction
};

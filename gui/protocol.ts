export type WebAction =
  | { t: 'quit' }
  | { t: 'drum' }
  | { t: 'setVolume', vol: number }
  | { t: 'setLowpassParam', lowp_param: number }
  | { t: 'setSequencer', inst: number, pat: number, on: boolean }
  ;

export type WebMessage = {
  message: WebAction
};

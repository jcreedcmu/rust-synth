export type WebAction =
  | { t: 'quit' }
  | { t: 'drum' }
  | { t: 'setVolume', vol: number }
  ;

export type WebMessage = {
  message: WebAction
};

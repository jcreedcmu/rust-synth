export type WebAction =
  | { t: 'quit' }
  | { t: 'drum' }
  | { t: 'setVolume' }
  ;

export type WebMessage = {
  message: WebAction
};

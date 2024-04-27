export type WebAction =
  | { t: 'quit' }
  | { t: 'drum' }
  ;

export type WebMessage = {
  message: WebAction
};

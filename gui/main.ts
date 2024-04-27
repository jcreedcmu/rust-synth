import { init } from "./app";

type WebAction =
  | 'Quit'
  | 'Drum'
  ;

type SynthMessage = {
  message: WebAction
};

function go() {

  const ws = new WebSocket('/ws/')
  ws.onopen = () => {
    console.log('ws opened on browser')
  }

  ws.onmessage = message => {
    console.log(`message received`, message.data);
  }

  function send(sm: SynthMessage) {
    ws.send(JSON.stringify(sm));
  }

  const action = document.getElementById('action')!;
  action.onmousedown = () => { send({ message: 'Drum' }); };

  const quit = document.getElementById('quit')!;
  quit.onmousedown = () => { send({ message: 'Quit' }); };

  init({});
}

go();

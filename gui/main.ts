import { init } from "./app";
import { WebMessage } from "./protocol";

function go() {

  const ws = new WebSocket('/ws/')
  ws.onopen = () => {
    console.log('ws opened on browser')
  }

  ws.onmessage = message => {
    console.log(`message received`, message.data);
  }

  function send(sm: WebMessage) {
    ws.send(JSON.stringify(sm));
  }

  const action = document.getElementById('action')!;
  action.onmousedown = () => { send({ message: { t: 'drum' } }); };

  const quit = document.getElementById('quit')!;
  quit.onmousedown = () => { send({ message: { t: 'quit' } }); };

  init({ send });
}

go();

import { init } from "./app";
import { WebMessage } from "./protocol";

function go() {

  let wsc: { ws: WebSocket } = { ws: new WebSocket('/ws/') };

  wsc.ws.onopen = () => {
    console.log('ws opened on browser')
  }

  wsc.ws.onclose = () => {
    console.log('ws closed on browser')
    setTimeout((() => {
      reconnect(wsc);
    }), 1500);
  }

  wsc.ws.onmessage = message => {
    console.log(`message received`, message.data);
  }

  function reconnect(wsc: { ws: WebSocket }) {
    console.log('retrying...')
    let ws = wsc.ws;
    var ws2 = new WebSocket(ws.url);
    ws2.onopen = ws.onopen;
    ws2.onmessage = ws.onmessage;
    ws2.onclose = ws.onclose;
    ws2.onerror = ws.onerror;
    wsc.ws = ws2;
  }


  function send(sm: WebMessage) {
    wsc.ws.send(JSON.stringify(sm));
  }

  const action = document.getElementById('action')!;
  action.onmousedown = () => { send({ message: { t: 'drum' } }); };

  const quit = document.getElementById('quit')!;
  quit.onmousedown = () => { send({ message: { t: 'quit' } }); };

  init({ send });
}

go();

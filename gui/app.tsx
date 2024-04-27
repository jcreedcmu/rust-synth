import { render, JSX } from 'preact';
import { useEffect, useRef, useState } from 'preact/hooks';
import { WebMessage } from './protocol';

type AppProps = {

};

export function init(props: AppProps) {
  render(<App {...props} />, document.querySelector('.app') as any);
}

type WebSocketContainer = { ws: WebSocket };

function App(props: AppProps): JSX.Element {
  const wsco = useRef<WebSocketContainer | undefined>(undefined);
  useEffect(() => {
    let wsc: WebSocketContainer = { ws: new WebSocket('/ws/') };

    wsc.ws.onopen = () => {
      console.log('ws opened on browser')
      wsco.current = wsc;
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
  }, []);

  function send(sm: WebMessage) {
    const wsc = wsco.current;
    if (wsc == undefined) {
      console.log('no websocket');
      return;
    }
    wsc.ws.send(JSON.stringify(sm));
  }

  const onInput = (e: Event) => {
    send({ message: { t: 'setVolume', vol: parseInt((e.target as HTMLInputElement).value) } });
  };
  const connected = false;
  return <div>
    <button onMouseDown={() => { send({ message: { t: 'drum' } }) }}>Action</button><br />
    <button onMouseDown={() => { send({ message: { t: 'quit' } }) }}>Quit</button><br />
    <input type="range" min="0" max="100" value="100" onInput={onInput} />
    {!connected ? <span style={{ backgroundColor: 'red', color: 'white' }}>DISCONNECTED<br /></span> : undefined}
  </div>;
}

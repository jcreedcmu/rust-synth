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
  const [connected, setConnected] = useState(true);
  const [drumVolume, setDrumVolume] = useState(100);
  const wsco = useRef<WebSocketContainer | undefined>(undefined);

  function reconnect(wsc: WebSocketContainer) {
    console.log('retrying...')
    let ws = wsc.ws;
    var ws2 = new WebSocket(ws.url);
    ws2.onopen = ws.onopen;
    ws2.onmessage = ws.onmessage;
    ws2.onclose = ws.onclose;
    ws2.onerror = ws.onerror;
    wsc.ws = ws2;
  }

  useEffect(() => {
    let wsc: WebSocketContainer = { ws: new WebSocket('/ws/') };

    wsc.ws.onopen = () => {
      setConnected(true);
      console.log('ws opened on browser')
      wsco.current = wsc;
    }

    wsc.ws.onclose = () => {
      setConnected(false);
      console.log('ws closed on browser')
    }

    wsc.ws.onmessage = message => {
      console.log(`message received`, message.data);
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
    const vol = parseInt((e.target as HTMLInputElement).value);
    send({ message: { t: 'setVolume', vol } });
    setDrumVolume(vol);
  };

  return <div>
    <button disabled={!connected} onMouseDown={() => { send({ message: { t: 'drum' } }) }}>Action</button><br />
    <button disabled={!connected} onMouseDown={() => { send({ message: { t: 'quit' } }) }}>Quit</button><br />
    <input disabled={!connected} type="range" min="0" max="100" value={drumVolume} onInput={onInput} />
    {!connected ? <span><br /><button style={{ backgroundColor: 'red', color: 'white' }}
      onClick={() => { reconnect(wsco.current!); }}>reconnect</button></span> : undefined}
  </div>;
}

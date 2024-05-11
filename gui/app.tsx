import ReactDOM from 'react-dom';
import { CSSProperties, useEffect, useRef, useState } from 'react';
import { LowpassControlBlock, WebMessage } from './protocol';
import { produce } from 'immer';
import { Chart } from './chart';

// Should match consts.rs
const BUS_DRY = 1;
const BUS_OUT = 0;

type AppProps = {

};

export function init(props: AppProps) {
  ReactDOM.render(<App {...props} />, document.querySelector('.app') as any);
}

type WebSocketContainer = { ws: WebSocket };

type SequencerProps = {
  send(msg: WebMessage): void;
}

function Sequencer(props: SequencerProps): JSX.Element {
  const initTable: boolean[][] = [
    [false, false], [false, false], [false, false], [false, false],
    [false, false], [false, false], [false, false], [false, false],
    [false, false], [false, false], [false, false], [false, false],
    [false, false], [false, false], [false, false], [false, false]
  ];
  const [table, setTable] = useState(initTable);
  function cellsOfRow(row: number): JSX.Element[] {
    let rv: JSX.Element[] = [];
    for (let i = 0; i < 16; i++) {
      const style: CSSProperties = {
        height: 20,
        width: 20,
        backgroundColor: table[i][row] ? 'black' : '#ddd'
      };
      function onClick(e: React.MouseEvent) {
        const oldVal = table[i][row];
        const newVal = !oldVal;
        setTable(produce(table, t => {
          t[i][row] = newVal;
        }));
        props.send({ message: { t: 'setSequencer', inst: row, on: newVal, pat: i } });
      }
      rv.push(<td><div style={style} onClick={onClick}></div></td>)
    }
    return rv;
  }
  const rows = [2, 1, 0].map(row => <tr>{cellsOfRow(row)}</tr>);
  return <table>{rows}</table>;
}

type LowpassCfgProps = {
  cfg: LowpassControlBlock,
  setLowpassCfg: (x: LowpassControlBlock) => void,
}

function LowpassCfg(props: LowpassCfgProps): JSX.Element[] {

  const setPos = (e: React.FormEvent, ix: number) => {
    const pos = parseInt((e.target as HTMLInputElement).value);
    const newCfg = produce(props.cfg, cfg => { cfg.taps[ix].pos = pos; });
    props.setLowpassCfg(newCfg);
  };

  const setWeight = (e: React.FormEvent, ix: number) => {
    const weight = parseInt((e.target as HTMLInputElement).value);
    const newCfg = produce(props.cfg, cfg => { cfg.taps[ix].weight = weight / 100; });
    props.setLowpassCfg(newCfg);
  };

  const taps = props.cfg.taps.map((tap, i) => {
    return <div>
      <input type="text" value={tap.pos} onInput={e => setPos(e, i)}></input>
      <input type="range" min="1" max="99" value={Math.round(tap.weight * 100)} onInput={e => setWeight(e, i)} />
    </div>;
  });

  return taps;
}

function App(props: AppProps): JSX.Element {
  const [connected, setConnected] = useState(true);
  const [drumVolume, setDrumVolume] = useState(100);
  const [lowpParam, setLowpParam] = useState(50);
  const [cfg, setCfg] = useState<LowpassControlBlock>({ selfWeight: 0.5, taps: [{ pos: 1, weight: 0.5 }] });
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
      console.log('ws opened on browser');
      wsco.current = wsc;
      send({
        message: {
          t: 'reconfigure', specs: [
            { t: 'midiManager', dst: BUS_DRY },
            { t: 'ugenGroup', dst: BUS_DRY },
            { t: 'lowPass', src: BUS_DRY, dst: BUS_OUT },
          ]
        }
      });
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

  const drumVolOnInput = (e: React.FormEvent) => {
    const vol = parseInt((e.target as HTMLInputElement).value);
    send({ message: { t: 'setVolume', vol } });
    setDrumVolume(vol);
  };

  const lowpParamOnInput = (e: React.FormEvent) => {
    const lowp_param = parseInt((e.target as HTMLInputElement).value);
    send({ message: { t: 'setLowpassParam', lowp_param: lowp_param / 100 } });
    setLowpParam(lowp_param);
  };

  function setLowpassCfg(cfg: LowpassControlBlock): void {
    const selfWeight = 1 - cfg.taps.map(x => x.weight).reduce((a, b) => a + b);
    const lowp_cfg = produce(cfg, cfg => {
      cfg.selfWeight = selfWeight;
    });
    setCfg(cfg);
    send({ message: { t: 'setLowpassConfig', lowp_cfg } });
  }

  return <div>
    <button disabled={!connected} onMouseDown={() => { send({ message: { t: 'drum' } }) }}>Action</button><br />
    <button disabled={!connected} onMouseDown={() => { send({ message: { t: 'quit' } }) }}>Quit</button><br />
    <input disabled={!connected} type="range" min="0" max="100" value={drumVolume} onInput={drumVolOnInput} />
    <LowpassCfg cfg={cfg} setLowpassCfg={setLowpassCfg} />
    {!connected ? <span><br /><button style={{ backgroundColor: 'red', color: 'white' }}
      onClick={() => { reconnect(wsco.current!); }}>reconnect</button></span> : undefined}
    <Sequencer send={send} />
    <br />
    <Chart lowp_param={lowpParam / 100} />
  </div>;
}

import ReactDOM from 'react-dom';
import { CSSProperties, useEffect, useRef, useState } from 'react';
import { ControlBlock, LowpassControlBlock, SynthMessage, WebMessage } from './protocol';
import { produce } from 'immer';
import { Chart } from './chart';
import { LowpassCfg, LowpassWidgetState } from './lowpass-widget';

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

const DEFAULT_DRUM_CONTROL_BLOCK: number = 0;
const DEFAULT_LOW_PASS_CONTROL_BLOCK: number = 1;
const DEFAULT_GAIN_CONTROL_BLOCK: number = 2;

function App(props: AppProps): JSX.Element {
  const [connected, setConnected] = useState(true);
  const [drumVolume, setDrumVolume] = useState(100);
  const [lowpParam, setLowpParam] = useState(50);
  const [meterValue, setMeterValue] = useState(0);
  const [cfg, setCfg] = useState<LowpassWidgetState>([{ pos: 1, weight: 90 }, { pos: 2620, weight: 10 }]);
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
            { t: 'meter', src: BUS_OUT },
          ]
        }
      });
    }

    wsc.ws.onclose = () => {
      setConnected(false);
      console.log('ws closed on browser')
    }

    wsc.ws.onmessage = message => {
      //   console.log(`message received`, message.data, typeof (message.data));
      try {
        const msg: SynthMessage = JSON.parse(message.data);
        if (msg.t == 'meter') {
          setMeterValue(msg.level);
        }
      } catch (e) {
        console.log(`couldn't parse ${message.data}`);
      }
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

  function setLowpassCfg(cfg: LowpassWidgetState): void {
    let taps = cfg.map(({ pos, weight }) => ({ pos, weight: weight / 100 }));
    let sum = taps.map(x => x.weight).reduce((a, b) => a + b);
    const minSelfWeight = 0.05;
    const maxSum = 1 - minSelfWeight;
    const s = maxSum / sum;
    if (sum > maxSum) {
      taps = taps.map(({ pos, weight }) => ({ pos, weight: weight * s }));
      sum = sum * s;
    }
    const selfWeight = 1 - sum;
    const ctl: ControlBlock = {
      t: 'Low',
      selfWeight,
      taps,
    };
    setCfg(cfg);
    send({ message: { t: 'setControlBlock', index: DEFAULT_LOW_PASS_CONTROL_BLOCK, ctl } });
  }
  //  <Chart lowp_param={0.50} />

  const meterDb = meterValue < 1e-10 ? '-infinity' : 20 * Math.log(meterValue) / Math.log(10);
  return <div>
    <button disabled={!connected} onMouseDown={() => { send({ message: { t: 'drum' } }) }}>Action</button><br />
    <button disabled={!connected} onMouseDown={() => { send({ message: { t: 'quit' } }) }}>Quit</button><br />
    <input disabled={!connected} type="range" min="0" max="100" value={drumVolume} onInput={drumVolOnInput} />
    <LowpassCfg cfg={cfg} setLowpassCfg={setLowpassCfg} />
    {!connected ? <span><br /><button style={{ backgroundColor: 'red', color: 'white' }}
      onClick={() => { reconnect(wsco.current!); }}>reconnect</button></span> : undefined}
    <Sequencer send={send} />
    <br />
    <b>RMS</b>: {meterDb}dB
  </div>;
}

import ReactDOM from 'react-dom';
import { CSSProperties, useEffect, useRef, useState } from 'react';
import { ControlBlock, LowpassControlBlock, SynthMessage, WebMessage } from './protocol';
import { produce } from 'immer';
import { Chart } from './chart';
import { LowpassCfg, LowpassWidgetState } from './lowpass-widget';

// Should match consts.rs
const BUS_OUT = 0;
const BUS_DRY = 1;
const BUS_PREGAIN = 2;

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
        props.send({ t: 'setSequencer', inst: row, on: newVal, pat: i });
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
  const [gain, setGain] = useState(10);
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
        t: 'reconfigure', specs: [
          { t: 'midiManager', dst: BUS_DRY },
          { t: 'ugenGroup', dst: BUS_DRY },
          { t: 'lowPass', src: BUS_DRY, dst: BUS_PREGAIN },
          { t: 'gain', src: BUS_PREGAIN, dst: BUS_OUT },
          { t: 'meter', src: BUS_OUT },
        ]
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

  const gainOnInput = (e: React.FormEvent) => {
    // interface_gain ranges from 1 to 99, so gain ranges from 0.1 to 9.9;
    const interface_gain = parseInt((e.target as HTMLInputElement).value);
    const gain = 10 * interface_gain / 100;
    const ctl: ControlBlock = { t: 'Gain', scale: gain };
    send({ t: 'setControlBlock', index: DEFAULT_GAIN_CONTROL_BLOCK, ctl });
    setGain(interface_gain);
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
    send({ t: 'setControlBlock', index: DEFAULT_LOW_PASS_CONTROL_BLOCK, ctl });
  }
  //  <Chart lowp_param={0.50} />

  const meterDb = meterValue < 1e-10 ? '-infinity' : 20 * Math.log(meterValue) / Math.log(10);
  return <div>
    <button disabled={!connected} onMouseDown={() => { send({ t: 'drum' }) }}>Action</button><br />
    <button disabled={!connected} onMouseDown={() => { send({ t: 'quit' }) }}>Quit</button><br />
    <input disabled={!connected} type="range" min="1" max="99" value={gain} onInput={gainOnInput} />
    <LowpassCfg cfg={cfg} setLowpassCfg={setLowpassCfg} />
    {!connected ? <span><br /><button style={{ backgroundColor: 'red', color: 'white' }}
      onClick={() => { reconnect(wsco.current!); }}>reconnect</button></span> : undefined}
    <Sequencer send={send} />
    <br />
    <b>RMS</b>: {meterDb}dB
  </div>;
}

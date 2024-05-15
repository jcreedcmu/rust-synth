import ReactDOM from 'react-dom';
import { CSSProperties, useEffect, useReducer, useRef, useState } from 'react';
import { ControlBlock, LowpassControlBlock, SynthMessage, Tap, WebMessage } from './protocol';
import { produce } from 'immer';
import { Chart } from './chart';
import { LowpassCfg, LowpassWidgetState } from './lowpass-widget';
import { useEffectfulReducer } from './use-effectful-reducer';

// Should match consts.rs
const BUS_OUT = 0;
const BUS_DRY = 1;
const BUS_PREGAIN = 2;
const BUS_PRELOW = 3;
const MAX_GAIN = 40;

type AppProps = {

};

export function init(props: AppProps) {
  ReactDOM.render(<App {...props} />, document.querySelector('.app') as any);
}

type WebSocketContainer = { ws: WebSocket };

export type Action =
  | { t: 'setSequencer', inst: number, pat: number, on: boolean }
  | { t: 'setConnected', connected: boolean }
  ;

type SequencerProps = {
  table: boolean[][],
  dispatch(action: Action): void;
}

export type State = {
  table: boolean[][],
  connected: boolean,
  outbox: WebMessage[],
}


type Effect =
  | { t: 'send', msg: WebMessage }
  ;


function reduce(state: State, action: Action): { state: State, effects: Effect[] } {
  let newState = reduce_inner(state, action);
  const effects: Effect[] = newState.outbox.map(msg => ({ t: 'send', msg }));
  newState = produce(newState, s => {
    s.outbox = [];
  });
  return { state: newState, effects };
}

function reduce_inner(state: State, action: Action): State {
  switch (action.t) {
    case 'setSequencer': {
      return produce(state, s => {
        s.table[action.pat][action.inst] = action.on;
        s.outbox.push(action);
      });
    }
    case 'setConnected': {
      return produce(state, s => { s.connected = action.connected; });
    }
  }
}

function mkState(): State {
  return {
    table: [
      [false, false], [false, false], [false, false], [false, false],
      [false, false], [false, false], [false, false], [false, false],
      [false, false], [false, false], [false, false], [false, false],
      [false, false], [false, false], [false, false], [false, false]
    ],
    outbox: [],
    connected: true,
  };
}

function Sequencer(props: SequencerProps): JSX.Element {
  const { table, dispatch } = props;
  function cellsOfInst(inst: number): JSX.Element[] {
    let rv: JSX.Element[] = [];
    for (let pat = 0; pat < 16; pat++) {
      const style: CSSProperties = {
        height: 20,
        width: 20,
        backgroundColor: table[pat][inst] ? 'black' : '#ddd'
      };
      function onClick(e: React.MouseEvent) {
        const oldVal = table[pat][inst];
        const on = !oldVal;
        dispatch({ t: 'setSequencer', pat, on, inst });
      }
      rv.push(<td><div style={style} onClick={onClick}></div></td>)
    }
    return rv;
  }
  const rows = [2, 1, 0].map(row => <tr>{cellsOfInst(row)}</tr>);
  return <table>{rows}</table>;
}

const DEFAULT_DRUM_CONTROL_BLOCK: number = 0;
const DEFAULT_LOW_PASS_CONTROL_BLOCK: number = 1;
const DEFAULT_GAIN_CONTROL_BLOCK: number = 2;
const DEFAULT_ALLPASS_CONTROL_BLOCK: number = 3;


export type Dispatch = (action: Action) => void;

function App(props: AppProps): JSX.Element {
  function doEffect(s: State, dispatch: Dispatch, e: Effect) {
    switch (e.t) {
      case 'send': send(e.msg); break;
    }
  }

  const [state, dispatch] = useEffectfulReducer<Action, State, Effect>(mkState(), reduce, doEffect);

  const [gain, setGain] = useState(10);
  const [highpassParam, setHighpassParam] = useState(50);
  const [allpassDelay, setAllpassDelay] = useState(50);
  const [allpassGain, setAllpassGain] = useState(50);
  const [allpassNaive, setAllpassNaive] = useState(true);
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
      dispatch({ t: 'setConnected', connected: true });
      console.log('ws opened on browser');
      wsco.current = wsc;
      send({
        t: 'reconfigure', specs: [
          { t: 'midiManager', dst: BUS_DRY },
          { t: 'ugenGroup', dst: BUS_DRY },
          { t: 'allPass', src: BUS_DRY, dst: BUS_PRELOW, ctl: DEFAULT_ALLPASS_CONTROL_BLOCK },
          { t: 'lowPass', src: BUS_PRELOW, dst: BUS_PREGAIN },
          { t: 'gain', src: BUS_PREGAIN, dst: BUS_OUT },
          { t: 'meter', src: BUS_OUT },
        ]
      });
    }

    wsc.ws.onclose = () => {
      dispatch({ t: 'setConnected', connected: false });
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
    const gain = MAX_GAIN * interface_gain / 100;
    const ctl: ControlBlock = { t: 'Gain', scale: gain };
    send({ t: 'setControlBlock', index: DEFAULT_GAIN_CONTROL_BLOCK, ctl });
    setGain(interface_gain);
  };

  const highpassOnInput = (e: React.FormEvent) => {
    // interface_gain ranges from 1 to 99, so gain ranges from 0.1 to 9.9;
    const interface_alpha = parseInt((e.target as HTMLInputElement).value);
    const alpha = interface_alpha / 100;
    const ctl: ControlBlock = {
      t: 'Low', taps: [
        { tp: { t: 'Rec' }, pos: 1, weight: alpha },
        { tp: { t: 'Input' }, pos: 0, weight: alpha },
        { tp: { t: 'Input' }, pos: 1, weight: -alpha },
      ]
    };
    send({ t: 'setControlBlock', index: DEFAULT_LOW_PASS_CONTROL_BLOCK, ctl });
    setHighpassParam(interface_alpha);
  };

  const allpassOnDelayInput = (e: React.FormEvent) => {
    const interface_param = parseInt((e.target as HTMLInputElement).value);
    setAllpassDelay(interface_param);
    const ctl: ControlBlock = {
      t: 'All',
      delay: interface_param,
      gain: allpassGain / 100,
      naive: allpassNaive,
    };
    send({ t: 'setControlBlock', index: DEFAULT_ALLPASS_CONTROL_BLOCK, ctl });
  };

  const allpassOnGainInput = (e: React.FormEvent) => {
    const interface_param = parseInt((e.target as HTMLInputElement).value);
    setAllpassGain(interface_param);
    const ctl: ControlBlock = {
      t: 'All',
      delay: allpassDelay,
      gain: interface_param / 100,
      naive: allpassNaive,
    };
    send({ t: 'setControlBlock', index: DEFAULT_ALLPASS_CONTROL_BLOCK, ctl });
  }

  const allpassOnNaiveInput = (e: React.FormEvent) => {
    const interface_param = !(e.target as HTMLInputElement).checked;
    setAllpassNaive(interface_param);
    const ctl: ControlBlock = {
      t: 'All',
      delay: allpassDelay,
      gain: allpassGain / 100,
      naive: interface_param,
    };
    send({ t: 'setControlBlock', index: DEFAULT_ALLPASS_CONTROL_BLOCK, ctl });
  }

  function setLowpassCfg(cfg: LowpassWidgetState): void {
    let taps: Tap[] = cfg.map(({ pos, weight }) => ({ pos, weight: weight / 100, tp: { t: 'Rec' } }));
    let sum = taps.map(x => x.weight).reduce((a, b) => a + b);
    const minSelfWeight = 0.05;
    const maxSum = 1 - minSelfWeight;
    const s = maxSum / sum;
    if (sum > maxSum) {
      taps = taps.map(({ pos, weight }) => ({ pos, weight: weight * s, tp: { t: 'Rec' } }));
      sum = sum * s;
    }
    const selfWeight = 1 - sum;
    const ctl: ControlBlock = {
      t: 'Low',
      taps: [...taps, { pos: 0, weight: selfWeight, tp: { t: 'Input' } }],
    };
    setCfg(cfg);
    send({ t: 'setControlBlock', index: DEFAULT_LOW_PASS_CONTROL_BLOCK, ctl });
  }
  //  <Chart lowp_param={0.50} />

  const meterDb = meterValue < 1e-10 ? '-infinity' : 20 * Math.log(meterValue) / Math.log(10);
  const { connected } = state;
  return <div>
    <button disabled={!connected} onMouseDown={() => { send({ t: 'drum' }) }}>Action</button><br />
    <button disabled={!connected} onMouseDown={() => { send({ t: 'quit' }) }}>Quit</button><br />
    <input disabled={!connected} type="range" min="1" max="99" value={gain} onInput={gainOnInput} />
    <LowpassCfg cfg={cfg} setLowpassCfg={setLowpassCfg} />
    {!connected ? <span><br /><button style={{ backgroundColor: 'red', color: 'white' }}
      onClick={() => { reconnect(wsco.current!); }}>reconnect</button></span> : undefined}
    <Sequencer dispatch={dispatch} table={state.table} />
    highpass: <input type="range" min="1" max="99" value={highpassParam} onInput={highpassOnInput} /><br />
    allpass delay: <input type="range" min="1" max="20000" value={allpassDelay} onInput={allpassOnDelayInput} /><br />
    allpass gain: <input type="range" min="1" max="99" value={allpassGain} onInput={allpassOnGainInput} /><br />
    allpass naive: <input type="checkbox" checked={allpassNaive} onInput={allpassOnNaiveInput} /><br />
    <br />
    <b>RMS</b>: {meterDb}dB
  </div>;
}

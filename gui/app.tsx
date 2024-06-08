import ReactDOM from 'react-dom';
import { CSSProperties, useEffect, useReducer, useRef, useState } from 'react';
import { Adsr, ControlBlock, LowpassControlBlock, MeterData, SynthMessage, Tap, WebMessage } from './protocol';
import { produce } from 'immer';
import { Chart } from './chart';
import { LowpassCfg, LowpassWidgetState } from './lowpass-widget';
import { useEffectfulReducer } from './use-effectful-reducer';
import { DbMeter } from './db-meter';

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
  | { t: 'setGain', iface_gain: number }
  | { t: 'setHighpass', iface_highpass: number }
  | { t: 'setAllpassDelay', iface_allpass_delay: number }
  | { t: 'setAllpassGain', iface_allpass_gain: number }
  | { t: 'setAllpassNaive', iface_allpass_naive: boolean }
  | { t: 'setMeterValues', msg: MeterData }
  | { t: 'setLowpassState', lowpassState: LowpassWidgetState }
  | { t: 'setText', text: string }
  | { t: 'setRoomSize', iface_roomsize: number }
  | { t: 'setWet', iface_wet: number }
  ;

type SequencerProps = {
  table: boolean[][],
  dispatch(action: Action): void;
}

export type AllpassState = {
  iface_allpass_gain: number,
  iface_allpass_delay: number,
  iface_allpass_naive: boolean,
};

export type State = {
  table: boolean[][],
  connected: boolean,
  iface_gain: number,
  iface_highpass: number,
  iface_roomsize: number,
  iface_wet: number,
  allpass: AllpassState;
  outbox: WebMessage[],
  meterData: MeterData,
  lowpassState: LowpassWidgetState,
  text: string,
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

function allpassMsg(a: AllpassState): WebMessage {
  const ctl: ControlBlock = {
    t: 'All',
    delay: a.iface_allpass_delay,
    gain: a.iface_allpass_gain / 100,
    naive: a.iface_allpass_naive,
  };
  return { t: 'setControlBlock', index: DEFAULT_ALLPASS_CONTROL_BLOCK, ctl };
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
    case 'setGain': {
      const gain = MAX_GAIN * action.iface_gain / 100;
      const ctl: ControlBlock = { t: 'Gain', scale: gain };
      const msg: WebMessage = { t: 'setControlBlock', index: DEFAULT_GAIN_CONTROL_BLOCK, ctl };
      return produce(state, s => {
        s.iface_gain = action.iface_gain;
        s.outbox.push(msg);
      });
    }
    case 'setHighpass': {
      const alpha = action.iface_highpass / 100;
      const ctl: ControlBlock = {
        t: 'Low', taps: [
          { tp: { t: 'Rec' }, pos: 1, weight: alpha },
          { tp: { t: 'Input' }, pos: 0, weight: alpha },
          { tp: { t: 'Input' }, pos: 1, weight: -alpha },
        ]
      };
      const msg: WebMessage = { t: 'setControlBlock', index: DEFAULT_LOW_PASS_CONTROL_BLOCK, ctl };
      return produce(state, s => {
        s.iface_highpass = action.iface_highpass;
        s.outbox.push(msg);
      });
    }
    case 'setAllpassDelay': {
      const { iface_allpass_delay } = action;
      const allpass = produce(state.allpass, a => {
        a.iface_allpass_delay = iface_allpass_delay;
      });
      const msg = allpassMsg(allpass);
      return produce(state, s => {
        s.allpass = allpass;
        s.outbox.push(msg);
      });
    }
    case 'setAllpassGain': {
      const { iface_allpass_gain } = action;
      const allpass = produce(state.allpass, a => {
        a.iface_allpass_gain = iface_allpass_gain;
      });
      const msg = allpassMsg(allpass);
      return produce(state, s => {
        s.allpass = allpass;
        s.outbox.push(msg);
      });
    }
    case 'setAllpassNaive': {
      const { iface_allpass_naive } = action;
      const allpass = produce(state.allpass, a => {
        a.iface_allpass_naive = iface_allpass_naive;
      });
      const msg = allpassMsg(allpass);
      return produce(state, s => {
        s.allpass = allpass;
        s.outbox.push(msg);
      });
    }
    case 'setMeterValues': {
      return produce(state, s => {
        s.meterData = action.msg;
      });
    }
    case 'setLowpassState': {
      const { lowpassState } = action;

      let taps: Tap[] = lowpassState.map(({ pos, weight }) => ({ pos, weight: weight / 100, tp: { t: 'Rec' } }));
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

      const msg: WebMessage = { t: 'setControlBlock', index: DEFAULT_LOW_PASS_CONTROL_BLOCK, ctl };

      return produce(state, s => {
        s.lowpassState = lowpassState;
        s.outbox.push(msg);
      });
    }
    case 'setText': {
      return produce(state, s => {
        s.text = action.text;
      });
    }
    case 'setRoomSize': {
      const newState = produce(state, s => {
        s.iface_roomsize = action.iface_roomsize;
      });
      const ctl: ControlBlock = {
        t: 'Reverb',
        roomSize: newState.iface_roomsize / 100,
        wet: newState.iface_wet / 100,
      };
      const msg: WebMessage = { t: 'setControlBlock', index: DEFAULT_REVERB_CONTROL_BLOCK, ctl };
      return produce(newState, s => {
        s.outbox.push(msg);
      });
    }
    case 'setWet': {
      const newState = produce(state, s => {
        s.iface_wet = action.iface_wet;
      });
      const ctl: ControlBlock = {
        t: 'Reverb',
        roomSize: newState.iface_roomsize / 100,
        wet: newState.iface_wet / 100,
      };
      const msg: WebMessage = { t: 'setControlBlock', index: DEFAULT_REVERB_CONTROL_BLOCK, ctl };
      return produce(newState, s => {
        s.outbox.push(msg);
      });
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
    iface_gain: 10,
    iface_highpass: 50,
    iface_roomsize: 50,
    iface_wet: 50,
    allpass: {
      iface_allpass_gain: 50,
      iface_allpass_delay: 50,
      iface_allpass_naive: true,
    },
    meterData: { level: 0, peak: 0 },
    lowpassState: [{ pos: 1, weight: 90 }, { pos: 2620, weight: 10 }],
    text: '',
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

const DEFAULT_REASONABLE_CONTROL_BLOCK = 0
const DEFAULT_DRUM_CONTROL_BLOCK: number = 10;
const DEFAULT_LOW_PASS_CONTROL_BLOCK: number = 1;
const DEFAULT_GAIN_CONTROL_BLOCK: number = 2;
const DEFAULT_ALLPASS_CONTROL_BLOCK: number = 3;
const DEFAULT_REVERB_CONTROL_BLOCK: number = 4;

function drum_adsr(dur_scale: number): Adsr {
  return {
    attack_s: 0.01 * dur_scale,
    decay_s: 0.05 * dur_scale,
    sustain: 0.2,
    release_s: 0.2 * dur_scale,
  };
}

export type Dispatch = (action: Action) => void;

function App(props: AppProps): JSX.Element {
  const [state, dispatch] = useEffectfulReducer<Action, State, Effect>(mkState(), reduce, doEffect);
  const wsco = useRef<WebSocketContainer | undefined>(undefined);

  function doEffect(s: State, dispatch: Dispatch, e: Effect) {
    switch (e.t) {
      case 'send': send(e.msg); break;
    }
  }

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
          { t: 'midiManager', dst: BUS_DRY, ci: DEFAULT_REASONABLE_CONTROL_BLOCK },
          { t: 'ugenGroup', dst: BUS_DRY },
          { t: 'allPass', src: BUS_DRY, dst: BUS_PRELOW, ci: DEFAULT_ALLPASS_CONTROL_BLOCK },
          //          { t: 'lowPass', src: BUS_PRELOW, dst: BUS_PREGAIN },
          { t: 'reverb', src: BUS_PRELOW, dst: BUS_PREGAIN, ci: DEFAULT_REVERB_CONTROL_BLOCK },
          { t: 'gain', src: BUS_PREGAIN, dst: BUS_OUT, ci: DEFAULT_GAIN_CONTROL_BLOCK },
          { t: 'meter', src: BUS_OUT },
        ]
      });
      send({
        t: 'setControlBlock', index: DEFAULT_DRUM_CONTROL_BLOCK, ctl: {
          t: 'Drum', vol: 1, adsr: drum_adsr(1.0), freq_hz: 660, freq2_hz: 1,
        }
      });
      send({
        t: 'setControlBlock', index: DEFAULT_DRUM_CONTROL_BLOCK + 1, ctl: {
          t: 'Drum', vol: 1, adsr: drum_adsr(0.5), freq_hz: 1760, freq2_hz: 1000,
        }
      });
      send({
        t: 'setControlBlock', index: DEFAULT_DRUM_CONTROL_BLOCK + 2, ctl: {
          t: 'Drum', vol: 1, adsr: drum_adsr(0.1), freq_hz: 6760, freq2_hz: 5000,
        }
      });

      send({
        t: 'setControlBlock', index: DEFAULT_REASONABLE_CONTROL_BLOCK, ctl: {
          t: 'Reasonable', adsr: {
            attack_s: 0.001,
            decay_s: 0.005,
            sustain: 0.3,
            release_s: 0.05,
          },
        }
      });

      send({
        t: 'setControlBlock', index: DEFAULT_LOW_PASS_CONTROL_BLOCK, ctl: {
          t: 'Low', taps: [
            { tp: { t: 'Input' }, pos: 0, weight: 0.5 },
            { tp: { t: 'Rec' }, pos: 1, weight: 0.5 },
          ],
        }
      });

      send({
        t: 'setControlBlock', index: DEFAULT_GAIN_CONTROL_BLOCK, ctl: {
          t: 'Gain', scale: 1.0,
        }
      });

      send({
        t: 'setControlBlock', index: DEFAULT_ALLPASS_CONTROL_BLOCK, ctl: {
          t: 'All', delay: 10, gain: 0.7, naive: true,
        }
      });

      send({
        t: 'setControlBlock', index: DEFAULT_REVERB_CONTROL_BLOCK, ctl: {
          t: 'Reverb',
          roomSize: 0.5,
          wet: 0.5,
        }
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
          dispatch({ t: 'setMeterValues', msg });
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
    // interface_gain ranges from 1 to 99, and gain ranges from ~0 to ~MAX_GAIN;
    dispatch({ t: 'setGain', iface_gain: parseInt((e.target as HTMLInputElement).value) });
  };


  const roomsizeOnInput = (e: React.FormEvent) => {
    dispatch({ t: 'setRoomSize', iface_roomsize: parseInt((e.target as HTMLInputElement).value) });
  };

  const wetOnInput = (e: React.FormEvent) => {
    dispatch({ t: 'setWet', iface_wet: parseInt((e.target as HTMLInputElement).value) });
  };

  const highpassOnInput = (e: React.FormEvent) => {
    dispatch({ t: 'setHighpass', iface_highpass: parseInt((e.target as HTMLInputElement).value) });
  };

  const textareaOnInput = (e: React.FormEvent) => {
    dispatch({ t: 'setText', text: (e.target as HTMLInputElement).value });
  };

  const updateText = (e: React.MouseEvent) => {
    send({ t: 'setText', text: state.text });
  };

  //  <Chart lowp_param={0.50} />

  const { connected, iface_gain, iface_highpass, allpass, meterData, text } = state;

  return <div>
    <button disabled={!connected} onMouseDown={() => { send({ t: 'drum' }) }}>Action</button><br />
    <button disabled={!connected} onMouseDown={() => { send({ t: 'quit' }) }}>Quit</button><br />
    <input disabled={!connected} type="range" min="1" max="99" value={iface_gain} onInput={gainOnInput} />
    <LowpassCfg
      cfg={state.lowpassState}
      setLowpassCfg={cfg => dispatch({ t: 'setLowpassState', lowpassState: cfg })}
    />

    {!connected ? <span><br /><button style={{ backgroundColor: 'red', color: 'white' }}
      onClick={() => { reconnect(wsco.current!); }}>reconnect</button></span> : undefined}

    <Sequencer dispatch={dispatch} table={state.table} />
    highpass: <input type="range" min="1" max="99" value={iface_highpass} onInput={highpassOnInput} /><br />
    allpass delay: <input type="range" min="1" max="20000" value={allpass.iface_allpass_delay}
      onInput={(e) => dispatch({ t: 'setAllpassDelay', iface_allpass_delay: parseInt((e.target as HTMLInputElement).value) })} />
    <br />
    allpass gain: <input type="range" min="1" max="99" value={allpass.iface_allpass_gain}
      onInput={(e) => dispatch({ t: 'setAllpassGain', iface_allpass_gain: parseInt((e.target as HTMLInputElement).value) })} />
    <br />
    allpass naive: <input type="checkbox" checked={allpass.iface_allpass_naive}
      onInput={(e) => dispatch({ t: 'setAllpassNaive', iface_allpass_naive: !((e.target as HTMLInputElement).checked) })} />
    <hr />


    room size: <input style={{ width: '90%' }} type="range" min="1" max="99" value={state.iface_roomsize} onInput={roomsizeOnInput} /><br />
    reverb wet: <input style={{ width: '90%' }} type="range" min="1" max="99" value={state.iface_wet} onInput={wetOnInput} /><br />

    <br />
    <br />
    <DbMeter label="RMS" value={meterData.level} /><br />
    <DbMeter label="Peak" value={meterData.peak} /><br />
    <textarea value={text} onInput={textareaOnInput}>d</textarea>
    <button onMouseDown={updateText}>update</button>
  </div>;
}

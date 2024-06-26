import { CSSProperties, useEffect, useRef } from 'react';
import ReactDOM from 'react-dom';
import { DbMeter } from './db-meter';
import { LowpassCfg } from './lowpass-widget';
import { Adsr, ControlBlock, SynthMessage, WebMessage } from './protocol';
import { RollEditor } from './roll';
import { Action, AllpassState, AppProps, Dispatch, Effect, State, WebSocketContainer, mkState } from './state';
import { useEffectfulReducer } from './use-effectful-reducer';
import { reduce } from './reduce';

// BUS_OUT should match consts.rs, rest are conventional
const BUS_OUT = 0;
export const MAX_GAIN = 40;

export function init(props: AppProps) {
  ReactDOM.render(<App {...props} />, document.querySelector('.app') as any);
}

export function allpassMsg(a: AllpassState): WebMessage {
  const ctl: ControlBlock = {
    t: 'All',
    delay: a.iface_allpass_delay,
    gain: a.iface_allpass_gain / 100,
    naive: a.iface_allpass_naive,
  };
  return { t: 'setControlBlock', index: DEFAULT_ALLPASS_CONTROL_BLOCK, ctl };
}

type SequencerProps = {
  table: boolean[][],
  dispatch(action: Action): void;
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
export const DEFAULT_LOW_PASS_CONTROL_BLOCK: number = 1;
export const DEFAULT_GAIN_CONTROL_BLOCK: number = 2;
const DEFAULT_ALLPASS_CONTROL_BLOCK: number = 3;
export const DEFAULT_REVERB_CONTROL_BLOCK: number = 4;

function drum_adsr(dur_scale: number): Adsr {
  return {
    attack_s: 0.01 * dur_scale,
    decay_s: 0.05 * dur_scale,
    sustain: 0.2,
    release_s: 0.2 * dur_scale,
  };
}

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

      const BUS_DRY = 1;
      const BUS_PREGAIN = 2;
      const BUS_PRELOW = 3;

      send({
        t: 'reconfigure', specs: [
          { t: 'midiManager', dst: BUS_DRY, ci: DEFAULT_REASONABLE_CONTROL_BLOCK },
          { t: 'ugenGroup', dst: BUS_DRY },
          //          { t: 'allPass', src: BUS_DRY, dst: BUS_PRELOW, ci: DEFAULT_ALLPASS_CONTROL_BLOCK },
          { t: 'reverb', src: BUS_DRY, dst: BUS_PRELOW, ci: DEFAULT_REVERB_CONTROL_BLOCK },
          { t: 'lowPass', src: BUS_PRELOW, dst: BUS_PREGAIN, ci: DEFAULT_LOW_PASS_CONTROL_BLOCK },
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

  //  <Chart lowp_param={0.50} />

  const { connected, iface_gain, iface_highpass, allpass, meterData, text } = state;

  const rollEditorProps = state.rollEditorState;

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
    <RollEditor {...rollEditorProps} dispatch={dispatch} />
  </div>;
}

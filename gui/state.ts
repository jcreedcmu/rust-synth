import { LowpassWidgetState } from './lowpass-widget';
import { MeterData, WebMessage } from './protocol';
import { RollEditorState } from './roll';
import { rollDims } from './roll-util';
import { score } from './score';
import { Point } from './types';

export type AppProps = {

};

export type WebSocketContainer = { ws: WebSocket };

export type RollAction =
  | { t: 'mousedown', p_in_canvas: Point }
  ;

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
  | { t: 'setRoomSize', iface_roomsize: number }
  | { t: 'setWet', iface_wet: number }
  | { t: 'Vscroll', top: number } // XXX move under rollAction
  | { t: 'rollAction', action: RollAction }
  ;

export type Dispatch = (action: Action) => void;

export type AllpassState = {
  iface_allpass_gain: number,
  iface_allpass_delay: number,
  iface_allpass_naive: boolean,
};

export type Effect =
  | { t: 'send', msg: WebMessage }
  ;

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
  rollEditorState: RollEditorState,
}
export function mkState(): State {
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
    rollEditorState: {
      offsetTicks: 0,
      debugOffsetTicks: 0,
      useOffsetTicks: 0,
      mouseState: { t: 'hover', mp: null },
      gridSize: 1,
      noteSize: 1,
      scrollOctave: 3,
      style: 'piano',
      pattern: score.patterns['default'],
      // XXX Fix this to be a Point:
      h: rollDims.w,
      w: rollDims.h,
      previewNote: null,
    }
  };
}

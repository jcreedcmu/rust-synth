import { produce } from 'immer';
import { ControlBlock, Tap, WebMessage } from './protocol';
import { Action, Effect, State } from './state';
import { MAX_GAIN, DEFAULT_GAIN_CONTROL_BLOCK, DEFAULT_LOW_PASS_CONTROL_BLOCK, allpassMsg, DEFAULT_REVERB_CONTROL_BLOCK } from './app';

export function reduce(state: State, action: Action): { state: State; effects: Effect[]; } {
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
    case 'Vscroll': {
      return produce(state, s => {
        s.rollEditorState.scrollOctave = action.top;
      });
    }
  }
}

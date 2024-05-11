import ReactDOM from 'react-dom';
import { CSSProperties, useEffect, useRef, useState } from 'react';
import { LowpassControlBlock, WebMessage } from './protocol';
import { produce } from 'immer';
import { Chart } from './chart';

export type InterfaceTap = {
  pos: number, // integer >0
  weight: number, // integer in [0, 99]
}

export type LowpassWidgetState = InterfaceTap[];

type LowpassCfgProps = {
  cfg: LowpassWidgetState,
  setLowpassCfg: (x: LowpassWidgetState) => void,
}

export function LowpassCfg(props: LowpassCfgProps): JSX.Element[] {

  const setPos = (e: React.FormEvent, ix: number) => {
    let pos = parseInt((e.target as HTMLInputElement).value);
    if (isNaN(pos))
      pos = 1;
    const newCfg = produce(props.cfg, cfg => { cfg[ix].pos = pos; });
    props.setLowpassCfg(newCfg);
  };

  const setWeight = (e: React.FormEvent, ix: number) => {
    const weight = parseInt((e.target as HTMLInputElement).value);
    const newCfg = produce(props.cfg, cfg => { cfg[ix].weight = weight; });
    props.setLowpassCfg(newCfg);
  };

  function onKeyDown(e: React.KeyboardEvent, ix: number) {
    console.log(e.code);
    if (e.code == 'ArrowUp') {
      const newCfg = produce(props.cfg, cfg => { cfg[ix].pos = cfg[ix].pos + 1; });
      props.setLowpassCfg(newCfg);
    }
    else if (e.code == 'ArrowDown') {
      const newCfg = produce(props.cfg, cfg => { cfg[ix].pos = Math.max(1, cfg[ix].pos - 1); });
      props.setLowpassCfg(newCfg);
    }
  }

  const taps = props.cfg.flatMap((tap, i) => {
    return [<br />, <div className="range-container">
      <input type="text"
        value={tap.pos}
        onKeyDown={e => onKeyDown(e, i)}
        onInput={e => setPos(e, i)} />
      <input type="range" min="1" max="99" value={tap.weight} onInput={e => setWeight(e, i)} />
    </div>];
  });

  return taps;
}

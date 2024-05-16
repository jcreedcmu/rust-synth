import { CSSProperties } from 'react';
import ReactDOM from 'react-dom';

export type DbProps = {
  value: number,
  label: string,
}

function dbOfLevel(x: number) {
  return x < 1e-10 ? '-infinity' : 20 * Math.log(x) / Math.log(10);
}

const PIX_PER_DB = 3;
export function DbMeter(props: DbProps): JSX.Element {
  const labelStyle: CSSProperties = {
    display: 'inline-block',
    margin: 1,
    width: 200,
    height: 20,
    whiteSpace: 'nowrap',
    overflowX: 'hidden',
  };
  const barWidth = props.value < 1e-10 ? 0 : 200 + PIX_PER_DB * 20 * Math.log(props.value) / Math.log(10);
  const divStyle: CSSProperties = {
    display: 'inline-block',
    border: '1px solid black',
    background: "linear-gradient(90deg, rgba(73,185,51,1) 164px, rgba(251,255,0,1) 182px, rgba(255,0,0,1) 200px)",
    margin: 1,
    width: barWidth,
  };
  return <span><div style={labelStyle}><b>{props.label}:</b> {dbOfLevel(props.value)} dB</div><div style={divStyle} >&nbsp;</div></span>;
}

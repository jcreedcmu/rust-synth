import ReactDOM from 'react-dom';

export type DbProps = {
  value: number,
}

function dbOfLevel(x: number) {
  return x < 1e-10 ? '-infinity' : 20 * Math.log(x) / Math.log(10);
}

export function DbMeter(props: DbProps): JSX.Element {
  return <span>{dbOfLevel(props.value)} dB</span>;
}

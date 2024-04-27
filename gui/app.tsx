import { render, JSX } from 'preact';
import { useState } from 'preact/hooks';
import { WebMessage } from './protocol';

type AppProps = {
  send: (msg: WebMessage) => void,
};

export function init(props: AppProps) {
  render(<App {...props} />, document.querySelector('.app') as any);
}

function App(props: AppProps): JSX.Element {
  const onInput = (e: Event) => {
    props.send({ message: { t: 'setVolume', vol: parseInt((e.target as HTMLInputElement).value) } });
  };
  return <div>
    <input type="range" min="0" max="100" value="100" onInput={onInput} />
  </div>;
}

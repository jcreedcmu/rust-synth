import { render, JSX } from 'preact';
import { useState } from 'preact/hooks';

type AppProps = {};

export function init(props: AppProps) {
  render(<App {...props} />, document.querySelector('.app') as any);
}

function App(props: AppProps): JSX.Element {
  const onInput = (e: Event) => {
    console.log((e.target as HTMLInputElement).value);
  };
  return <div>
    <input type="range" min="1" max="100" value="50" onInput={onInput} />
  </div>;
}

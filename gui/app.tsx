import { render, JSX } from 'preact';
import { useState } from 'preact/hooks';

type AppProps = {};

export function init(props: AppProps) {
  render(<App {...props} />, document.querySelector('.app') as any);
}

function App(props: AppProps): JSX.Element {
  return <span>hello</span>;
}

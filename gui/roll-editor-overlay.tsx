import { RollEditorProps } from "./roll";
import { CanvasInfo, useCanvas } from "./use-canvas";

export type RollEditorOverlayProps = RollEditorProps;

// XXX cut down rolleditoroverlayprops to what's necessary
function render(ci: CanvasInfo, state: RollEditorOverlayProps) {
  const { d } = ci;
}

export function RollEditorOverlay(props: RollEditorOverlayProps): JSX.Element {
  const deps = [props];
  function onLoad() { }

  const [cref, mc] = useCanvas(props, render, deps, onLoad);
  return <canvas style={{ position: 'absolute', width: props.w, height: props.h }} ref={cref} />
}

import { RollEditorProps } from "./roll";
import { CanvasInfo, useCanvas } from "./use-canvas";

export type RollEditorOverlayProps = RollEditorProps;

// XXX cut down rolleditoroverlayprops to what's necessary
function render(ci: CanvasInfo, state: RollEditorOverlayProps) {
  const { d } = ci;
  d.fillRect(0, 0, ci.size.x, ci.size.y);
}

export function RollEditorOverlay(props: RollEditorOverlayProps): JSX.Element {
  const deps = [props];
  function onLoad() { }

  const [cref, mc] = useCanvas(props, render, deps, onLoad);
  return <canvas style={{ width: props.w, height: props.h }} ref={cref} />
}

import { RollEditorProps } from "./roll";
import { CanvasInfo, useCanvas } from "./use-canvas";

export type RollEditorMainProps = RollEditorProps & { scroll: number };

// XXX cut down rolleditormainprops to what's necessary
function render(ci: CanvasInfo, state: RollEditorMainProps) {
  const { d } = ci;
  d.fillRect(0, 0, ci.size.x, ci.size.y);
}

export function RollEditorMain(props: RollEditorMainProps): JSX.Element {
  const deps = [props];
  function onLoad() { }

  const [cref, mc] = useCanvas(props, render, deps, onLoad);
  return <canvas style={{ width: props.w, height: props.h }} ref={cref} />
}

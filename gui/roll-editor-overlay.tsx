import { RollDispatch, RollEditorProps, RollEditorState } from "./roll";
import { CanvasInfo, useCanvas } from "./use-canvas";
import { rrelpos } from "./dutil";

export type RollEditorOverlayProps = RollEditorState & { dispatch: RollDispatch };

// XXX cut down rolleditoroverlayprops to what's necessary
function render(ci: CanvasInfo, state: RollEditorOverlayProps) {
  const { d } = ci;
}

export function RollEditorOverlay(props: RollEditorOverlayProps): JSX.Element {
  const deps = [props];
  const { dispatch } = props;
  function onLoad() { }

  const [cref, mc] = useCanvas(props, render, deps, onLoad);
  return <canvas style={{ position: 'absolute', width: props.w, height: props.h }}
    ref={cref}
    onMouseDown={e => dispatch({ t: 'Mousedown', p_in_canvas: rrelpos(e) })}
  />
}

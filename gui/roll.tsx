import { Pattern, Note } from "./types";
import { mpoint } from "./roll-util";
import { useCanvas, CanvasInfo } from './use-canvas';

export type Style = "piano" | "drums";

export type RollMouseState =
  | { t: "hover", mp: mpoint | null }
  | { t: "down", orig: mpoint, now: mpoint | null }
  | {
    t: "resizeNote", fromRight: boolean, orig: mpoint, now: mpoint | null,
    note: Note, noteIx: number
  }

export type DerivedState = {
  // XXX this belongs scoped to editpattern mode data I think?
  previewNote: Note | null,
}

export type RollEditorProps = {
  offsetTicks: number | null,
  debugOffsetTicks: number | null,
  useOffsetTicks: number,
  mouseState: RollMouseState,
  gridSize: number,
  noteSize: number,
  scrollOctave: number,
  style: Style,
  pattern: Pattern,
} & DerivedState & { w: number, h: number };

function render(ci: CanvasInfo, state: RollEditorProps) {
  const { d } = ci;
  d.fillRect(0, 0, ci.size.x, ci.size.y);
}

export function RollEditor(props: RollEditorProps): JSX.Element {
  function onLoad() { }
  const deps = [props];
  const [cref, mc] = useCanvas(props, render, deps, onLoad);
  return <canvas style={{ width: props.w, height: props.h }} ref={cref} />
}

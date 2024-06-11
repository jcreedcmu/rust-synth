import { CSSProperties } from "react";
import { RollEditorMain } from "./roll-editor-main";
import { RollEditorOverlay } from "./roll-editor-overlay";
import { mpoint } from "./roll-util";
import { Dispatch } from "./state";
import { Note, Pattern } from "./types";

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

export type RollEditorState = {
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

export type RollEditorProps = RollEditorState & { dispatch: Dispatch };


export function RollEditor(props: RollEditorProps): JSX.Element {
  const { dispatch } = props;

  const vscroller = <div></div>; // XXX
  const hscroller = <div></div>; // XXX

  const style: CSSProperties = {
    width: props.w, height: props.h,
    position: "relative", display: "inline-block"
  };

  const cursorState = props.mouseState.t == "resizeNote" ? "resize" : undefined;
  function goBack() {
    //// XXX go back to editing song
    // dispatch({ t: "EditSong" })
  }
  const elt = <div>
    <img className="button" src="img/back.png" onClick={goBack}></img><br />
    <div style={style} className={cursorState} >
      <RollEditorMain {...props} scroll={0} />
      <RollEditorOverlay {...props} />
      {vscroller}
      {hscroller}
    </div>
  </div>;
  return elt;

}

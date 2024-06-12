import { CSSProperties } from "react";
import { RollEditorMain } from "./roll-editor-main";
import { RollEditorOverlay, RollEditorOverlayProps } from "./roll-editor-overlay";
import { GUTTER_WIDTH, PIANO_WIDTH, RollAction, RollMouseState, getScrollbarDims, mpoint } from "./roll-util";
import { Action, Dispatch } from "./state";
import { Note, Pattern } from "./types";
import { VScrollBar } from "./vscrollbar";

export type Style = "piano" | "drums";

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

export type RollDispatch = (action: RollAction) => void;
export type RollEditorProps = RollEditorState & { dispatch: Dispatch }; // XXX change this to RollDispatch


export function RollEditor(props: RollEditorProps): JSX.Element {
  const { w, h, dispatch } = props;

  const onVscroll = (top: number) => dispatch({ t: "Vscroll", top: Math.round(3 * top / h) });

  const vscroller = <VScrollBar height={h}
    scrollTop={props.scrollOctave * h / 3}
    onScroll={onVscroll}
    content_height={(7 / 3) * h}
    x={w} y={0} />;

  const s = getScrollbarDims();

  // XXX this still doesn't actually do anything
  const hscroller =
    <div style={{
      height: s.height, top: h, left: PIANO_WIDTH + GUTTER_WIDTH,
      width: w - PIANO_WIDTH - GUTTER_WIDTH, overflowY: 'hidden',
      overflowX: 'scroll', position: 'absolute'
    }}>
      <div style={{ width: 2 * w }}>&nbsp;</div>
    </div>;

  const style: CSSProperties = {
    width: props.w, height: props.h,
    position: "relative", display: "inline-block"
  };

  const cursorState = props.mouseState.t == "resizeNote" ? "resize" : undefined;
  function goBack() {
    //// XXX go back to editing song
    // dispatch({ t: "EditSong" })
  }
  const overlayProps: RollEditorOverlayProps = { ...props, dispatch: action => dispatch({ t: 'rollAction', action }) };
  const elt = <div>
    <img className="button" src="img/back.png" onClick={goBack}></img><br />
    <div style={style} className={cursorState} >
      <RollEditorMain {...props} scroll={0} />
      <RollEditorOverlay {...overlayProps} />
      {vscroller}
      {hscroller}
    </div>
  </div>;
  return elt;

}

import { RollDispatch, RollEditorProps, RollEditorState } from "./roll";
import { CanvasInfo, useCanvas } from "./use-canvas";
import { rrelpos } from "./dutil";
import { FAT_PIXELS_PER_TICK, GUTTER_WIDTH, PIANO_OCTAVE_VSPACE, PIANO_WIDTH, SCALE, get_camera, inset, note_name, rect_of_note } from "./roll-util";

export type RollEditorOverlayProps = RollEditorState & { dispatch: RollDispatch };

// XXX cut down rolleditoroverlayprops to what's necessary
function render(ci: CanvasInfo, props: RollEditorOverlayProps) {
  const { d } = ci;
  const { w, h } = props;

  /* if (this.w != props.w || this.h != props.h)
   *   this.setDims(props.w, props.h); */

  d.clearRect(0, 0, w, h);
  if (props.previewNote != null) {
    const rect = rect_of_note(props.previewNote, get_camera(props.scrollOctave));
    d.fillStyle = "white";
    d.textAlign = "right";
    d.textBaseline = "middle";
    d.font = "bold 10px sans-serif ";

    const pitch = props.previewNote.pitch;
    const annot = (props.style == "drums" ? pitch : note_name[pitch % 12]) as string;
    d.fillText(annot, rect[0] - 1, rect[1] + 1 + rect[3] / 2);
    d.fillRect.apply(d, rect);
    d.clearRect.apply(d, inset(rect));
  }

  // draw playback cursor
  if (props.offsetTicks != null) {
    const relToUse = props.offsetTicks - props.useOffsetTicks;
    if (relToUse >= 0 && relToUse < props.pattern.length) {
      d.fillStyle = "white";
      d.fillRect(PIANO_WIDTH + GUTTER_WIDTH + SCALE * FAT_PIXELS_PER_TICK * (props.offsetTicks - props.useOffsetTicks), 0,
        2, PIANO_OCTAVE_VSPACE * 3);
    }
  }
}

export function RollEditorOverlay(props: RollEditorOverlayProps): JSX.Element {
  const deps = [props];
  const { dispatch } = props;
  function onLoad() { }

  const [cref, mc] = useCanvas(props, render, deps, onLoad);
  return <canvas style={{ position: 'absolute', width: props.w, height: props.h }}
    ref={cref}
    onMouseDown={e => dispatch({ t: 'Mousedown', p_in_canvas: rrelpos(e) })}
    onMouseMove={e => dispatch({ t: 'Mousemove', p_in_canvas: rrelpos(e) })}
    onMouseUp={e => dispatch({ t: 'Mouseup' })}
    onMouseLeave={e => dispatch({ t: 'Mouseleave' })}
  />
}

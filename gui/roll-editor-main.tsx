import { RollEditorProps, Style } from "./roll";
import { BLACK_NOTE_WIDTH, DARKER_DARK_GRAY, FAT_PIXELS_PER_TICK, GUTTER_WIDTH, LIGHTER_DARK_GRAY, PIANO_H, PIANO_OCTAVE_VSPACE, PIANO_W, PIANO_WIDTH, PITCH_HEIGHT, PIXELS_PER_TICK, SCALE, noteColors, drawBox, get_camera, keytype, rect_of_note, Camera, inset } from "./roll-util";
import { Note } from "./types";
import { CanvasInfo, useCanvas } from "./use-canvas";

export type RollEditorMainProps = RollEditorProps & { scroll: number };

function draw_piano_octave(d: CanvasRenderingContext2D, x: number, y: number) {
  d.save();
  d.translate(x, y);
  drawBox(d, 0, 0, PIANO_W, PIANO_H, 1, "#f8f8d8", "black");
  d.fillStyle = "black";
  [14, 28, 42, 56, 69, 83].forEach(wks =>
    d.fillRect(0, wks * SCALE, PIANO_W * SCALE, 1 * SCALE)
  );
  [1, 3, 5, 8, 10].forEach(bk =>
    drawBox(d, 0, PITCH_HEIGHT * bk, BLACK_NOTE_WIDTH, PITCH_HEIGHT + 1, 1, "#2e2234", "black")
  );

  d.restore();
}

function draw_gutter(d: CanvasRenderingContext2D, x: number, y: number, w: number, style: Style) {
  d.fillStyle = "black";
  d.save();
  d.translate(x, y);
  drawBox(d, 0, 0, w, PIANO_H, 1, LIGHTER_DARK_GRAY, "black")

  if (style == "piano") {
    for (let n = 0; n < 12; n++) {
      if (keytype[n]) {
        drawBox(d, w - 7, PITCH_HEIGHT * n, 5, PITCH_HEIGHT + 1, 1, DARKER_DARK_GRAY, "black");
      }
    }
  }

  d.restore();
}

function draw_staff_octave(d: CanvasRenderingContext2D, x: number, y: number, w: number, style: Style, gridSize: number) {
  const effectiveGridSize = 4; // enh... 'visibleGridSize'? ignores gridSize argument.
  d.fillStyle = "black";
  d.save();
  d.translate(x, y);
  d.fillRect(0, 0, w * SCALE, PIANO_H * SCALE);
  for (let n = 0; n < 12; n++) {
    d.fillStyle = keytype[n] || style == "drums" ? DARKER_DARK_GRAY : LIGHTER_DARK_GRAY;
    d.fillRect(SCALE, (PITCH_HEIGHT * n + 1) * SCALE, (w - 2) * SCALE, (PITCH_HEIGHT - 1) * SCALE);
  }
  d.fillStyle = "black";
  for (let n = 0; n * PIXELS_PER_TICK * effectiveGridSize < SCALE * w; n++) {
    d.fillRect(n * PIXELS_PER_TICK * effectiveGridSize, 0, SCALE, PIANO_H * SCALE);
  }
  d.restore();
}

function draw_notes(d: CanvasRenderingContext2D, notes: Note[], camera: Camera) {
  notes.forEach(note => {
    const r = rect_of_note(note, camera);
    d.fillStyle = "black";
    d.fillRect.apply(d, r);
    d.fillStyle = noteColors[note.pitch % 12];
    d.fillRect.apply(d, inset(r));
  });
}

// XXX cut down rolleditormainprops to what's necessary
function render(ci: CanvasInfo, props: RollEditorMainProps) {
  const { pattern, scrollOctave } = props;
  const { notes, length } = pattern;
  const { d } = ci;
  d.fillStyle = LIGHTER_DARK_GRAY;
  d.fillRect(0, 0, props.w, props.h);
  for (let oc = 0; oc < 3; oc++) {
    if (props.style == "piano") {
      draw_piano_octave(d, 0, oc * PIANO_OCTAVE_VSPACE);
    }
    draw_gutter(d, PIANO_WIDTH + SCALE, oc * PIANO_OCTAVE_VSPACE, 10, props.style);
    draw_staff_octave(d, PIANO_WIDTH + GUTTER_WIDTH, 0 + oc * PIANO_OCTAVE_VSPACE, length * FAT_PIXELS_PER_TICK + 1, props.style, props.gridSize);
  }
  draw_notes(d, notes, get_camera(scrollOctave));
}

export function RollEditorMain(props: RollEditorMainProps): JSX.Element {
  const deps = [props];
  function onLoad() { }

  const [cref, mc] = useCanvas(props, render, deps, onLoad);
  return <canvas style={{ position: 'absolute', width: props.w, height: props.h }} ref={cref} />
}

import { Note, Point, Rect } from './types'

export const SCALE = 2; // units: pixels per fat pixel
export const PIANO_H = 97;
export const PIANO_W = 58;
export const PIANO_OCTAVE_VSPACE = (PIANO_H - 1) * SCALE;
export const PIANO_WIDTH = (PIANO_W) * SCALE;
export const GUTTER_W = 8;
export const GUTTER_WIDTH = GUTTER_W * SCALE;
export const SCORE_W = 250;
export const SCORE_WIDTH = SCORE_W * SCALE;
export const FAT_PIXELS_PER_TICK = 6;
export const PIXELS_PER_TICK = FAT_PIXELS_PER_TICK * SCALE;
export const PITCH_HEIGHT = 8;
export const BLACK_NOTE_WIDTH = 34;

export const LIGHTER_DARK_GRAY = "#262626";
export const DARKER_DARK_GRAY = "#141414";

export const rollDims = {
  w: PIANO_WIDTH + GUTTER_WIDTH + SCORE_WIDTH,
  h: PIANO_OCTAVE_VSPACE * 3 + SCALE
};
export type RollMouseState =
  | { t: "hover", mp: mpoint | null }
  | { t: "down", orig: mpoint, now: mpoint | null }
  | {
    t: "resizeNote", fromRight: boolean, orig: mpoint, now: mpoint | null,
    note: Note, noteIx: number
  }

export type RollMode = {
  t: "editPattern",
  patName: string,
  mouseState: RollMouseState,

  // when editing a pattern, there is still a weaker sense in which we
  // are editing a particular use of that pattern, for the purpose of
  // showing the playback cursor.
  useOffsetTicks: number,
}

// XXX rename 'time' to 'ticks'
// XXX rename 'mpoint' to 'Mpoint'
export type mpoint = { pitch: number, time: number } & Point // point also in "musical coordinates"

export function drawBox(d: CanvasRenderingContext2D, x: number, y: number, w: number, h: number, border: number, c: string, bc: string) {
  d.fillStyle = bc;
  d.fillRect(x * SCALE, y * SCALE, w * SCALE, h * SCALE);
  d.fillStyle = c;
  d.fillRect((x + border) * SCALE, (y + border) * SCALE, (w - 2 * border) * SCALE, (h - 2 * border) * SCALE);
}

// 0 for white key, 1 for black key
export const keytype = [0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0];

export function y0pitch_of_scrollOctave(scrollOctave: number) {
  return 12 * (9 - scrollOctave) - 1;
}

export function get_camera(scrollOctave: number): Point {
  return {
    x: PIANO_WIDTH + GUTTER_WIDTH,
    y: y0pitch_of_scrollOctave(scrollOctave) * PITCH_HEIGHT * SCALE
  };
}

export const noteColors = [
  "#7882e2",
  "#38396e",
  "#df4f48",
  "#696800",
  "#fffd58",
  "#f47937",
  "#782a00",
  "#71d256",
  "#790061",
  "#d343b6",
  "#075152",
  "#75c4c5",
];

export type Camera = Point;

export function rect_of_note(n: Note, c: Camera): Rect {
  return [c.x + n.time[0] * PIXELS_PER_TICK,
  c.y - n.pitch * PITCH_HEIGHT * SCALE,
  (n.time[1] - n.time[0]) * PIXELS_PER_TICK + SCALE,
  SCALE * (PITCH_HEIGHT + 1)];
}

export function inset(rect: Rect): Rect {
  return [rect[0] + SCALE, rect[1] + SCALE, rect[2] - 2 * SCALE, rect[3] - 2 * SCALE];
}

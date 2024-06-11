export type Point = { x: number, y: number };
export type Note = { pitch: number, time: [number, number] };

export type IdNote = Note & { id: string };

export type Pattern = {
  length: number,
  notes: IdNote[],
};

// XXX fix this, this is terrible
export type Rect = [number, number, number, number]; // x y w h, in canvas pixels

export type PatUse = {
  lane: number,
  patName: string,
  start: number,
  duration: number,
}

export type Song = PatUse[]

export type Score = {
  next_id: number,
  duration: number, // ticks
  seconds_per_tick: number,
  loop_start: number,
  loop_end: number,
  song: Song,
  patterns: { [P in string]: Pattern },
};

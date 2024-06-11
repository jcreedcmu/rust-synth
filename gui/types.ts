export type Point = { x: number, y: number };
export type Note = { pitch: number, time: [number, number] };

export type IdNote = Note & { id: string };

export type Pattern = {
  length: number,
  notes: IdNote[],
};

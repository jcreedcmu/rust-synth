import { Note, Point } from './types'

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

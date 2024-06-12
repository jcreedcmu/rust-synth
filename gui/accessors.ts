import { produce } from "immer";
import { RollEditorState } from "./roll";
import { IdNote } from "./types";

export function getCurrentNotes(state: RollEditorState): IdNote[] {
  return state.pattern.notes;
}

export function updateCurrentNotes(state: RollEditorState, f: (x: IdNote[]) => IdNote[]): RollEditorState {
  const newNotes = f(state.pattern.notes);
  return produce(state, s => {
    s.pattern.notes = newNotes;
  });
}

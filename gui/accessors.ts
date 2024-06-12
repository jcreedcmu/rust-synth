import { RollEditorState } from "./roll";
import { IdNote } from "./types";

export function getCurrentNotes(state: RollEditorState): IdNote[] {
  return state.pattern.notes;
}

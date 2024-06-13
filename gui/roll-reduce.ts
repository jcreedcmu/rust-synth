import { RollEditorState } from './roll';
import { produce } from 'immer';
import { GUTTER_WIDTH, PIANO_WIDTH, PITCH_HEIGHT, PIXELS_PER_TICK, RollAction, RollMouseAction, RollMouseState, SCALE, mpoint, restrictAtState, snapToGrid, y0pitch_of_scrollOctave } from './roll-util';
import { findLast, findLastIndex, unreachable } from './util';
import { getCurrentNotes, getCurrentPat, updateCurrentNotes } from './accessors';
import { IdNote, Note, Point } from './types';
import { augment_and_snap } from './extra-util';

function find_note_at_mpoint<T extends Note>(notes: T[], mp: mpoint): T | undefined {
  return findLast(notes, note => {
    return (note.pitch == mp.pitch
      && note.time[0] <= mp.time
      && note.time[1] >= mp.time);
  });
}

function rollReduceMouse(state: RollEditorState, ms: RollMouseState, a: RollMouseAction): RollEditorState {
  const notes = getCurrentNotes(state);
  switch (ms.t) {
    case "down":
      if (a.t == "Mouseup") {
        console.log('up a down');
        const mp = ms.orig;
        console.log('notes', notes, 'mp', mp);
        const note = find_note_at_mpoint(notes, mp);
        console.log('note', note);
        if (note !== undefined) {
          // Delete note
          const notIt = (x: IdNote) => x.id != note.id;
          return produce(updateCurrentNotes(state, n => n.filter(notIt)), s => {
            // Record the duration of that note as the default note length, in case we want to
            // place the "same" note elsewhere maybe
            s.noteSize = note.time[1] - note.time[0];
          });
        }
        else {
          // Create note
          const sn: Note | null = restrictAtState(snapToGrid(state.gridSize, state.noteSize, mp), state);
          if (sn == null)
            return state
          else {
            const id = state.next_id;
            return produce(updateCurrentNotes(state, n => n.concat([{ ...sn, id: "n" + id }])), s => {
              s.next_id = id + 1;
            });
          }
          return state;
        }
        return state;
      }
      break;
    case "resizeNote":
      if (a.t == "Mousemove") {
        if (ms.now == null) return state;
        const oldLength = (ms.note.time[1] - ms.note.time[0]);
        const lengthDiff = augment_and_snap(ms.now.time - ms.orig.time);
        if (ms.fromRight) {
          const newLength = Math.max(1, lengthDiff + oldLength);
          const pat = getCurrentPat(state);
          if (pat == undefined)
            return state;
          const newEnd = Math.min(pat.length, ms.note.time[0] + newLength);

          return produce(updateCurrentNotes(state, notes => produce(notes, n => { n[ms.noteIx].time[1] = newEnd; })),
            s => { s.noteSize = newLength; });
        }
        else {
          const newLength = Math.max(1, oldLength - lengthDiff);
          const newBegin = Math.max(0, ms.note.time[1] - newLength);

          return produce(updateCurrentNotes(state, notes => produce(notes, n => { n[ms.noteIx].time[0] = newBegin; })),
            s => { s.noteSize = newLength; });
        }
        return state;
      }
      else {
        return state;
      }
  }
  return state;
}

function xd_of_ticksd(ticksd: number): number {
  return ticksd * PIXELS_PER_TICK;
}

function find_note_index_at_mpoint(notes: Note[], mp: mpoint): number {
  return findLastIndex(notes, note => {
    return (note.pitch == mp.pitch
      && note.time[0] <= mp.time
      && note.time[1] >= mp.time);
  });
}

function mpoint_from_canvas(point_in_canvas: Point, scrollOctave: number): mpoint {
  return {
    ...point_in_canvas,
    pitch: y0pitch_of_scrollOctave(scrollOctave) - Math.floor(point_in_canvas.y / (SCALE * PITCH_HEIGHT)),
    time: (point_in_canvas.x - (PIANO_WIDTH + GUTTER_WIDTH + SCALE)) / PIXELS_PER_TICK,
  };
}

function rollNewMouseState(state: RollEditorState, ms: RollMouseState, a: RollMouseAction): RollMouseState {
  const notes = getCurrentNotes(state);
  function m_of_c(x: Point) { return mpoint_from_canvas(x, state.scrollOctave); }
  switch (ms.t) {
    case "hover":
      switch (a.t) {
        case "Mousemove": return { t: "hover", mp: m_of_c(a.p_in_canvas) };
        case "Mousedown": const mp = m_of_c(a.p_in_canvas); return { t: "down", orig: mp, now: mp };
        case "Mouseup": return ms; // this happens for mouse events that started outside the editor
        case "Mouseleave": return { ...ms, mp: null };
      }

    case "down":
      switch (a.t) {
        case "Mousemove": {
          const pa: mpoint = ms.orig;
          const pb: mpoint = m_of_c(a.p_in_canvas);
          let rv: RollMouseState = { t: "down", orig: pa, now: pb };
          if (xd_of_ticksd(Math.abs(pa.time - pb.time)) > 5) {
            const noteIx = find_note_index_at_mpoint(notes, pa);
            if (noteIx != -1) {
              const note = notes[noteIx];
              const fromRight = pa.time > (note.time[0] + note.time[1]) / 2;
              rv = { t: "resizeNote", fromRight, orig: pa, now: pb, note, noteIx };
            }
          }
          return rv;
        }
        case "Mousedown": throw "impossible";
        case "Mouseup": return { t: "hover", mp: ms.now };
        case "Mouseleave": return { ...ms, now: null };
      }

    case "resizeNote":
      switch (a.t) {
        case "Mousemove": return { ...ms, now: m_of_c(a.p_in_canvas) };
        case "Mousedown": throw "impossible";
        case "Mouseup": return { t: "hover", mp: ms.now };
        case "Mouseleave": return { ...ms, now: null };
      }
  }
}

export function rollReduce(state: RollEditorState, action: RollAction): RollEditorState {
  const nst = rollReduceMouse(state, state.mouseState, action);
  const nmst = rollNewMouseState(state, state.mouseState, action);
  return produce(nst, s => {
    s.mouseState = nmst;
    // s.mode = { ...mode, mouseState: nmst }; // XXX save this for later once I implement app modes
  });
}

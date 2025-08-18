import { highlightRange as highlightEffect } from './editor/active-block.js';
import type { EditorView } from '@codemirror/view';

// Current editor instance used for synchronisation.
let currentView: EditorView | null = null;

/**
 * Register the CodeMirror view used for highlighting.
 */
export function registerEditor(view: EditorView) {
  currentView = view;
}

/**
 * Highlight a range in the editor defined by an anchor.
 *
 * The anchor is expected to be a tuple [from, to] representing
 * character offsets inside the document. If the anchor is `null`
 * or invalid the current highlight is cleared.
 *
 * @param anchor Tuple containing start and end positions or null to clear
 * @returns `true` if a range was highlighted, otherwise `false`
 */
export function highlightRange(anchor: [number, number] | null): boolean {
  if (!currentView) return false;
  if (!anchor || anchor.length !== 2) {
    currentView.dispatch({ effects: [highlightEffect.of(null)] });
    return false;
  }
  const [from, to] = anchor;
  currentView.dispatch({
    effects: [highlightEffect.of({ from, to })],
    selection: { anchor: from, head: to },
    scrollIntoView: true,
  });
  return true;
}

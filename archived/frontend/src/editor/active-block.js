import { StateEffect, StateField } from "@codemirror/state";
import { EditorView, Decoration } from "@codemirror/view";
import { indentation } from "@codemirror/language";

// Effect used to highlight a range in the editor
export const highlightRange = StateEffect.define({
  map: (value, mapping) => ({
    from: mapping.mapPos(value.from),
    to: mapping.mapPos(value.to)
  })
});

const highlightField = StateField.define({
  create() {
    return Decoration.none;
  },
  update(highlight, tr) {
    highlight = highlight.map(tr.changes);
    for (const effect of tr.effects) {
      if (effect.is(highlightRange)) {
        if (effect.value && typeof effect.value.from === "number" && typeof effect.value.to === "number") {
          return Decoration.set([
            Decoration.mark({ class: "cm-active-block" }).range(effect.value.from, effect.value.to)
          ]);
        }
        return Decoration.none;
      }
    }
    return highlight;
  },
  provide: f => EditorView.decorations.from(f)
});

function findBlockRange(state, pos) {
  const doc = state.doc;
  const line = doc.lineAt(pos);
  let baseIndent = indentation(state, line.from);
  if (baseIndent < 0) baseIndent = 0;

  let start = line.from;
  for (let l = line.number - 1; l > 0; l--) {
    const prev = doc.line(l);
    if (!prev.text.trim()) { start = prev.from; continue; }
    const ind = indentation(state, prev.from);
    if (ind == null || ind < baseIndent) break;
    start = prev.from;
  }

  let end = line.to;
  for (let l = line.number + 1; l <= doc.lines; l++) {
    const next = doc.line(l);
    if (!next.text.trim()) { end = next.to; continue; }
    const ind = indentation(state, next.from);
    if (ind == null || ind < baseIndent) break;
    end = next.to;
  }
  return { from: start, to: end };
}

const blockHighlighter = EditorView.updateListener.of(update => {
  if (update.selectionSet || update.docChanged) {
    const pos = update.state.selection.main.head;
    const range = findBlockRange(update.state, pos);
    update.view.dispatch({ effects: highlightRange.of(range) });
  }
});

const blockTheme = EditorView.baseTheme({
  ".cm-active-block": { backgroundColor: "rgba(255, 235, 59, 0.3)" }
});

export const activeBlock = [highlightField, blockHighlighter, blockTheme];

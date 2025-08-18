import { StateField, RangeSetBuilder } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";

// Regular expression to match TODO or FIXME words
export const TASK_RE = /\b(?:TODO|FIXME)\b/g;

function buildDecorations(doc) {
  const builder = new RangeSetBuilder();
  for (let i = 1; i <= doc.lines; i++) {
    const line = doc.line(i);
    let m;
    TASK_RE.lastIndex = 0;
    while ((m = TASK_RE.exec(line.text))) {
      const from = line.from + m.index;
      const to = from + m[0].length;
      builder.add(from, to, Decoration.mark({ class: "cm-todo" }));
    }
  }
  return builder.finish();
}

export function extractTasks(doc) {
  const tasks = [];
  for (let i = 1; i <= doc.lines; i++) {
    const line = doc.line(i);
    let m;
    TASK_RE.lastIndex = 0;
    while ((m = TASK_RE.exec(line.text))) {
      tasks.push({ line: i, from: line.from + m.index, text: line.text.trim() });
    }
  }
  return tasks;
}

export function updateTodoPanel(view) {
  const list = document.getElementById("todo-list");
  if (!list) return;
  const tasks = extractTasks(view.state.doc);
  list.innerHTML = "";
  tasks.forEach(t => {
    const opt = document.createElement("option");
    opt.value = String(t.from);
    opt.textContent = `${t.line}: ${t.text}`;
    list.appendChild(opt);
  });
  list.onchange = e => {
    const pos = parseInt(e.target.value, 10);
    if (!isNaN(pos)) {
      view.dispatch({ selection: { anchor: pos }, scrollIntoView: true });
    }
  };
}

const todoField = StateField.define({
  create(state) {
    return buildDecorations(state.doc);
  },
  update(deco, tr) {
    if (tr.docChanged) {
      return buildDecorations(tr.state.doc);
    }
    return deco.map(tr.changes);
  },
  provide: f => EditorView.decorations.from(f)
});

const todoTheme = EditorView.baseTheme({
  ".cm-todo": {
    backgroundColor: "rgba(255, 229, 100, 0.3)",
    outline: "1px solid orange"
  }
});

export const todoHighlight = [
  todoField,
  todoTheme,
  EditorView.updateListener.of(update => {
    if (update.docChanged) updateTodoPanel(update.view);
  })
];

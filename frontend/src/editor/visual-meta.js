import { StateField, RangeSetBuilder } from "https://cdn.jsdelivr.net/npm/@codemirror/state@6.4.0/dist/index.js";
import { Decoration, EditorView } from "https://cdn.jsdelivr.net/npm/@codemirror/view@6.21.3/dist/index.js";

const templates = {
  rust: () => `// @VISUAL_META ${JSON.stringify({id: crypto.randomUUID(), x:0, y:0})}`,
  javascript: () => `// @VISUAL_META ${JSON.stringify({id: crypto.randomUUID(), x:0, y:0})}`,
  python: () => `# @VISUAL_META ${JSON.stringify({id: crypto.randomUUID(), x:0, y:0})}`,
  html: () => `<!-- @VISUAL_META ${JSON.stringify({id: crypto.randomUUID(), x:0, y:0})} -->`,
  css: () => `/* @VISUAL_META ${JSON.stringify({id: crypto.randomUUID(), x:0, y:0})} */`,
};

export function insertVisualMeta(view, lang) {
  const tmpl = (templates[lang] || templates.javascript)() + "\n";
  const { from } = view.state.selection.main;
  view.dispatch({ changes: { from, to: from, insert: tmpl } });
}

export function updateMetaComment(view, meta) {
  const text = view.state.doc.toString();
  const re = /@VISUAL_META\s*(\{[^\}]*\})/g;
  let m;
  while ((m = re.exec(text)) !== null) {
    const json = m[1];
    try {
      const obj = JSON.parse(json);
      if (obj.id === meta.id) {
        obj.x = meta.x;
        obj.y = meta.y;
        const newJson = JSON.stringify(obj);
        const start = m.index + m[0].indexOf(json);
        const end = start + json.length;
        view.dispatch({ changes: { from: start, to: end, insert: newJson } });
        return true;
      }
    } catch(_) {
      // ignore
    }
  }
  return false;
}

const regexes = [
  /#\s*@VISUAL_META\s*(\{.*\})/g,
  /\/\/\s*@VISUAL_META\s*(\{.*\})/g,
  /\/\*\s*@VISUAL_META\s*(\{.*?\})\s*\*\//gs,
  /<!--\s*@VISUAL_META\s*(\{.*?\})\s*-->/gs,
];

export const visualMetaHighlighter = StateField.define({
  create() {
    return Decoration.none;
  },
  update(deco, tr) {
    if (!tr.docChanged) return deco;
    const builder = new RangeSetBuilder();
    const text = tr.newDoc.toString();
    regexes.forEach(re => {
      re.lastIndex = 0;
      let m;
      while ((m = re.exec(text)) !== null) {
        const json = m[1];
        try {
          JSON.parse(json);
        } catch (_) {
          const start = m.index + m[0].indexOf(json);
          const end = start + json.length;
          builder.add(start, end, Decoration.mark({ class: "cm-invalid-meta" }));
        }
      }
    });
    return builder.finish();
  },
  provide: f => EditorView.decorations.from(f)
});


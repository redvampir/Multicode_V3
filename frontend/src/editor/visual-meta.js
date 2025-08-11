import { StateField, RangeSetBuilder } from "https://cdn.jsdelivr.net/npm/@codemirror/state@6.4.0/dist/index.js";
import { Decoration, EditorView } from "https://cdn.jsdelivr.net/npm/@codemirror/view@6.21.3/dist/index.js";
import { hoverTooltip } from "https://cdn.jsdelivr.net/npm/@codemirror/language@6.10.1/dist/index.js";
import schema from "./visual-meta-schema.json" with { type: "json" };

const tmplObj = () => ({
  id: crypto.randomUUID(),
  x: 0,
  y: 0,
  tags: [],
  updated_at: new Date().toISOString(),
});
const templates = {
  rust: () => `// @VISUAL_META ${JSON.stringify(tmplObj())}`,
  javascript: () => `// @VISUAL_META ${JSON.stringify(tmplObj())}`,
  python: () => `# @VISUAL_META ${JSON.stringify(tmplObj())}`,
  html: () => `<!-- @VISUAL_META ${JSON.stringify(tmplObj())} -->`,
  css: () => `/* @VISUAL_META ${JSON.stringify(tmplObj())} */`,
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
        if (Array.isArray(meta.tags)) {
          obj.tags = meta.tags;
        }
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

function validateAgainstSchema(obj, sch, path = "") {
  const errors = [];
  if (sch.type === "object") {
    if (typeof obj !== "object" || obj === null || Array.isArray(obj)) {
      errors.push(`${path || 'value'} should be object`);
      return errors;
    }
    if (sch.required) {
      for (const key of sch.required) {
        if (!(key in obj)) errors.push(`${path}${key} is required`);
      }
    }
    if (sch.additionalProperties === false && sch.properties) {
      for (const key of Object.keys(obj)) {
        if (!(key in sch.properties)) errors.push(`${path}${key} is not allowed`);
      }
    }
    for (const [key, val] of Object.entries(sch.properties || {})) {
      if (key in obj) {
        errors.push(...validateAgainstSchema(obj[key], val, `${path}${key}.`));
      }
    }
  } else if (sch.type === "number") {
    if (typeof obj !== "number") errors.push(`${path.slice(0, -1)} should be number`);
  } else if (sch.type === "string") {
    if (typeof obj !== "string") errors.push(`${path.slice(0, -1)} should be string`);
  } else if (sch.type === "array") {
    if (!Array.isArray(obj)) {
      errors.push(`${path.slice(0, -1)} should be array`);
    } else if (sch.items) {
      obj.forEach((item, i) => {
        errors.push(...validateAgainstSchema(item, sch.items, `${path}${i}.`));
      });
    }
  }
  return errors;
}

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
        let start = m.index + m[0].indexOf(json);
        let end = start + json.length;
        let obj;
        try {
          obj = JSON.parse(json);
        } catch (_) {
          builder.add(start, end, Decoration.mark({ class: "cm-invalid-meta", attributes: { title: "Invalid JSON" } }));
          continue;
        }
        const errors = validateAgainstSchema(obj, schema);
        if (errors.length) {
          builder.add(start, end, Decoration.mark({ class: "cm-invalid-meta", attributes: { title: errors.join("; ") } }));
        }
      }
    });
    return builder.finish();
  },
  provide: f => EditorView.decorations.from(f)
});

export const visualMetaTooltip = hoverTooltip((view, pos) => {
  const text = view.state.doc.toString();
  for (const re of regexes) {
    re.lastIndex = 0;
    let m;
    while ((m = re.exec(text)) !== null) {
      const json = m[1];
      const start = m.index + m[0].indexOf(json);
      const end = start + json.length;
      if (pos >= start && pos <= end) {
        const dom = document.createElement("div");
        for (const [key, prop] of Object.entries(schema.properties)) {
          const line = document.createElement("div");
          line.textContent = `${key}: ${prop.description || ""}`;
          dom.appendChild(line);
        }
        return { pos: start, end, above: true, create: () => ({ dom }) };
      }
    }
  }
  return null;
});


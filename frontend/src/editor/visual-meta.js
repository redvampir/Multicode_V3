import { StateField, RangeSetBuilder } from "@codemirror/state";
import { Decoration, EditorView } from "@codemirror/view";
import { hoverTooltip } from "@codemirror/language";
import schema from "./meta.schema.json" with { type: "json" };

const tmplObj = () => ({
  id: crypto.randomUUID(),
  version: 1,
  x: 0,
  y: 0,
  tags: [],
  links: [],
  updated_at: new Date().toISOString(),
});

// Map id -> start position of JSON object inside the meta block
export const metaPositions = new Map();

function getMetaBlock(text) {
  const start = text.indexOf("/* @VISUAL_META");
  if (start === -1) return null;
  const end = text.indexOf("*/", start);
  if (end === -1) return null;
  return { start, end: end + 2 };
}

function ensureMetaBlock(view) {
  let text = view.state.doc.toString();
  let block = getMetaBlock(text);
  if (!block) {
    const insert = "\n/* @VISUAL_META\n*/";
    const pos = text.length;
    view.dispatch({ changes: { from: pos, to: pos, insert } });
    text = view.state.doc.toString();
    block = getMetaBlock(text);
  }
  return block;
}

function rebuildMetaPositions(text) {
  metaPositions.clear();
  const block = getMetaBlock(text);
  if (!block) return;
  const contentStart = text.indexOf("\n", block.start) + 1;
  let pos = contentStart;
  const content = text.slice(contentStart, block.end - 2);
  const lines = content.split("\n");
  for (const line of lines) {
    const trimmed = line.trim();
    if (trimmed) {
      try {
        const obj = JSON.parse(trimmed);
        if (obj.id) {
          const offset = pos + line.indexOf(trimmed);
          metaPositions.set(obj.id, offset);
        }
      } catch (_) {
        // ignore invalid lines
      }
    }
    pos += line.length + 1;
  }
}

export function insertVisualMeta(view, _lang) {
  const meta = tmplObj();
  const marker = `// @VISUAL_META ${meta.id}\n`;
  const { from } = view.state.selection.main;
  view.dispatch({ changes: { from, to: from, insert: marker } });

  const block = ensureMetaBlock(view);
  const insertPos = block.end - 2; // before closing */
  const jsonLine = `${JSON.stringify(meta)}\n`;
  view.dispatch({ changes: { from: insertPos, to: insertPos, insert: jsonLine } });

  rebuildMetaPositions(view.state.doc.toString());
}

export function updateMetaComment(view, meta) {
  let pos = metaPositions.get(meta.id);
  if (pos == null) {
    rebuildMetaPositions(view.state.doc.toString());
    pos = metaPositions.get(meta.id);
  }
  if (pos == null) return false;

  const text = view.state.doc.toString();
  const end = text.indexOf("\n", pos);
  const json = text.slice(pos, end === -1 ? undefined : end);
  try {
    const obj = JSON.parse(json);
    if (typeof obj.version !== "number") {
      obj.version = 1;
    }
    if (typeof meta.x === "number") obj.x = meta.x;
    if (typeof meta.y === "number") obj.y = meta.y;
    if (Array.isArray(meta.tags)) {
      obj.tags = meta.tags;
    }
    if (Array.isArray(meta.links)) {
      obj.links = meta.links;
    }
    obj.updated_at = new Date().toISOString();
    const newJson = JSON.stringify(obj);
    view.dispatch({ changes: { from: pos, to: pos + json.length, insert: newJson } });
    rebuildMetaPositions(view.state.doc.toString());
    return true;
  } catch (_) {
    return false;
  }
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


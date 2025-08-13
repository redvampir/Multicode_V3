import { StateField, RangeSetBuilder } from "@codemirror/state";
import * as viewPkg from "@codemirror/view";
const { Decoration, EditorView } = viewPkg;
let ViewPlugin;
try {
  ViewPlugin = viewPkg.ViewPlugin;
} catch (_) {
  ViewPlugin = null;
}
import { hoverTooltip, foldEffect } from "@codemirror/language";
import settings from "../../settings.json";
import schema from "./meta.schema.json";

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

let currentView = null;

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

function extractMetaCoords(text) {
  const map = new Map();
  const block = getMetaBlock(text);
  if (!block) return map;
  const contentStart = text.indexOf("\n", block.start) + 1;
  const content = text.slice(contentStart, block.end - 2);
  const lines = content.split("\n");
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      const obj = JSON.parse(trimmed);
      if (obj.id) map.set(obj.id, { x: obj.x, y: obj.y });
    } catch (_) {
      // ignore
    }
  }
  return map;
}

function highlightMetaById(view, id) {
  let pos = metaPositions.get(id);
  if (pos == null) {
    rebuildMetaPositions(view.state.doc.toString());
    pos = metaPositions.get(id);
  }
  if (pos == null) return;
  const text = view.state.doc.toString();
  const end = text.indexOf("\n", pos);
  const to = end === -1 ? text.length : end;
  view.dispatch({ selection: { anchor: pos, head: to }, scrollIntoView: true });
}

export function scrollToMeta(id) {
  if (currentView) {
    highlightMetaById(currentView, id);
  }
}

export function foldMetaBlock(view) {
  if (settings?.editor?.autoFoldMeta === false) return;
  const block = getMetaBlock(view.state.doc.toString());
  if (!block) return;
  view.dispatch({ effects: foldEffect.of({ from: block.start, to: block.end }) });
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

export function reorderMeta(ids) {
  if (!currentView || !Array.isArray(ids)) return;
  const view = currentView;
  const text = view.state.doc.toString();
  const block = getMetaBlock(text);
  if (!block) return;
  const contentStart = text.indexOf("\n", block.start) + 1;
  const contentEnd = block.end - 2;
  const content = text.slice(contentStart, contentEnd);
  const lines = content.split("\n");
  const items = lines
    .map(line => {
      const trimmed = line.trim();
      let id = null;
      try {
        const obj = JSON.parse(trimmed);
        if (obj.id) id = obj.id;
      } catch (_) {
        /* ignore */
      }
      return { line, id, used: false };
    })
    .filter(i => i.line.trim());
  const newLines = [];
  ids.forEach(id => {
    const it = items.find(i => i.id === id && !i.used);
    if (it) {
      newLines.push(it.line);
      it.used = true;
    }
  });
  items.forEach(it => {
    if (!it.used) newLines.push(it.line);
  });
  const newContent = newLines.join("\n") + (newLines.length ? "\n" : "");
  view.dispatch({ changes: { from: contentStart, to: contentEnd, insert: newContent } });
  rebuildMetaPositions(view.state.doc.toString());
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

export const visualMetaMessenger = ViewPlugin && ViewPlugin.fromClass ? ViewPlugin.fromClass(class {
  constructor(view) {
    this.view = view;
    currentView = view;
    this.onMessage = this.onMessage.bind(this);
    this.onClick = this.onClick.bind(this);
    this.onMouseMove = this.onMouseMove.bind(this);
    this.onMouseLeave = this.onMouseLeave.bind(this);
    this.lastHoverId = null;
    this.lastMouse = { x: 0, y: 0 };
    this.prevCoords = extractMetaCoords(view.state.doc.toString());
    this.debounceTimer = null;
    this.tooltip = document.createElement('div');
    this.tooltip.style.position = 'fixed';
    this.tooltip.style.pointerEvents = 'none';
    this.tooltip.style.background = '#333';
    this.tooltip.style.color = '#fff';
    this.tooltip.style.padding = '4px 8px';
    this.tooltip.style.borderRadius = '4px';
    this.tooltip.style.display = 'none';
    document.body.appendChild(this.tooltip);
    window.addEventListener('message', this.onMessage);
    view.dom.addEventListener('click', this.onClick);
    view.dom.addEventListener('mousemove', this.onMouseMove);
    view.dom.addEventListener('mouseleave', this.onMouseLeave);
  }
  destroy() {
    window.removeEventListener('message', this.onMessage);
    this.view.dom.removeEventListener('click', this.onClick);
    this.view.dom.removeEventListener('mousemove', this.onMouseMove);
    this.view.dom.removeEventListener('mouseleave', this.onMouseLeave);
    document.body.removeChild(this.tooltip);
    clearTimeout(this.debounceTimer);
    if (currentView === this.view) currentView = null;
  }
  scheduleSync() {
    clearTimeout(this.debounceTimer);
    this.debounceTimer = setTimeout(() => this.syncCoords(), 300);
  }
  syncCoords() {
    const text = this.view.state.doc.toString();
    rebuildMetaPositions(text);
    const current = extractMetaCoords(text);
    for (const [id, meta] of current.entries()) {
      const prev = this.prevCoords.get(id);
      if (!prev || prev.x !== meta.x || prev.y !== meta.y) {
        if (typeof meta.x === 'number' || typeof meta.y === 'number') {
          window.postMessage({ source: 'visual-meta', type: 'updatePos', id, x: meta.x, y: meta.y }, '*');
        }
      }
    }
    this.prevCoords = current;
  }
  update(update) {
    if (update.docChanged) this.scheduleSync();
  }
  onMessage(e) {
    const { source, id, type, kind, color, thumbnail, ids } = e.data || {};
    if (source === 'visual-canvas') {
      if (type === 'block-info' && id === this.lastHoverId) {
        if (thumbnail) {
          this.tooltip.innerHTML = `<img src="${thumbnail}" />`;
        } else {
          this.tooltip.textContent = `${kind || ''} ${color || ''}`.trim();
        }
        this.tooltip.style.left = this.lastMouse.x + 10 + 'px';
        this.tooltip.style.top = this.lastMouse.y + 10 + 'px';
        this.tooltip.style.display = 'block';
      } else if (type === 'reorder' && Array.isArray(ids) && settings?.visual?.syncOrder !== false) {
        reorderMeta(ids);
      } else if (id) {
        highlightMetaById(this.view, id);
      }
    }
  }
  onClick(e) {
    const pos = this.view.posAtCoords({ x: e.clientX, y: e.clientY });
    if (pos == null) return;
    const text = this.view.state.doc.toString();
    rebuildMetaPositions(text);
    let clickedId = null;
    for (const [id, start] of metaPositions.entries()) {
      const end = text.indexOf("\n", start);
      const to = end === -1 ? text.length : end;
      if (pos >= start && pos <= to) {
        clickedId = id;
        break;
      }
    }
    if (clickedId) {
      window.postMessage({ source: 'visual-meta', id: clickedId }, '*');
      highlightMetaById(this.view, clickedId);
    }
  }
  onMouseMove(e) {
    this.lastMouse = { x: e.clientX, y: e.clientY };
    const pos = this.view.posAtCoords({ x: e.clientX, y: e.clientY });
    if (pos == null) {
      this.lastHoverId = null;
      this.tooltip.style.display = 'none';
      return;
    }
    const text = this.view.state.doc.toString();
    rebuildMetaPositions(text);
    let hoverId = null;
    for (const [id, start] of metaPositions.entries()) {
      const end = text.indexOf("\n", start);
      const to = end === -1 ? text.length : end;
      if (pos >= start && pos <= to) {
        hoverId = id;
        break;
      }
    }
    if (hoverId) {
      if (hoverId !== this.lastHoverId) {
        this.lastHoverId = hoverId;
        window.postMessage({ source: 'visual-meta', type: 'request-block-info', id: hoverId }, '*');
      }
      if (this.tooltip.style.display !== 'none') {
        this.tooltip.style.left = this.lastMouse.x + 10 + 'px';
        this.tooltip.style.top = this.lastMouse.y + 10 + 'px';
      }
    } else {
      this.lastHoverId = null;
      this.tooltip.style.display = 'none';
    }
  }
  onMouseLeave() {
    this.lastHoverId = null;
    this.tooltip.style.display = 'none';
  }
}) : [];


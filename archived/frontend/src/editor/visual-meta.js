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
import { setDiagnostics } from "@codemirror/lint";
import settings from "../../settings.json";
import schema from "./meta.schema.json";
import { parsePatch } from "diff";
import { emit, on } from "../shared/event-bus.js";

let invoke = async () => {};
if (typeof window !== "undefined" && window.__TAURI__?.invoke) {
  invoke = window.__TAURI__.invoke;
}

const tmplObj = () => ({
  id: crypto.randomUUID(),
  version: 1,
  x: 0,
  y: 0,
  tags: [],
  links: [],
  tests: [],
  updated_at: new Date().toISOString(),
  history: [],
});

// Map id -> start position of JSON object inside the meta block
export const metaPositions = new Map();

// Map of block id -> lint message
export const lintMessages = new Map();

let currentView = null;

export function editorDiagnostics(view, errors) {
  if (!view) return;
  const diagnostics = [];
  const doc = view.state.doc;
  const entries = errors instanceof Map ? errors.entries() : Object.entries(errors || {});
  for (const [id, msg] of entries) {
    const pos = metaPositions.get(id);
    if (pos == null) continue;
    const line = doc.lineAt(pos);
    diagnostics.push({ from: line.from, to: line.to, severity: "error", message: msg });
  }
  setDiagnostics(view, diagnostics);
}

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

export function listMetaIds(text) {
  rebuildMetaPositions(text);
  return Array.from(metaPositions.keys());
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

function findIdForPos(text, pos) {
  const marker = "// @VISUAL_META";
  const start = text.lastIndexOf(marker, pos);
  if (start === -1) return null;
  const end = text.indexOf("\n", start);
  const line = text.slice(start, end === -1 ? text.length : end);
  const m = line.match(/\/\/\s*@VISUAL_META\s+([A-Za-z0-9_-]+)/);
  return m ? m[1] : null;
}

function highlightRangeBetweenIds(view, fromId, toId) {
  const text = view.state.doc.toString();
  const positions = findInlineMetaPositions(text);
  const fromPos = positions.get(fromId);
  const toPos = positions.get(toId);
  if (!fromPos || !toPos) {
    emit('edgeNotFound', { from: fromId, to: toId });
    return;
  }
  let first = fromPos;
  let second = toPos;
  if (first.start > second.start) {
    [first, second] = [second, first];
  }
  const start = text.indexOf("\n", first.end) + 1;
  const end = second.start;
  view.dispatch({ selection: { anchor: start, head: end }, scrollIntoView: true });
}

export function foldMetaBlock(view) {
  if (settings?.editor?.autoFoldMeta === false) return;
  const block = getMetaBlock(view.state.doc.toString());
  if (!block) return;
  view.dispatch({ effects: foldEffect.of({ from: block.start, to: block.end }) });
}

export function insertVisualMeta(view, _lang, pos) {
  const meta = tmplObj();
  const marker = `// @VISUAL_META ${meta.id}\n`;
  const from = typeof pos === "number" ? pos : view.state.selection.main.from;
  view.dispatch({ changes: { from, to: from, insert: marker } });

  const block = ensureMetaBlock(view);
  const insertPos = block.end - 2; // before closing */
  const jsonLine = `${JSON.stringify(meta)}\n`;
  view.dispatch({ changes: { from: insertPos, to: insertPos, insert: jsonLine } });

  rebuildMetaPositions(view.state.doc.toString());
  return meta;
}

export function refreshBlockCount(view) {
  const count = extractMetaCoords(view.state.doc.toString()).size;
  const el = document.getElementById("block-count");
  if (el) el.textContent = String(count);
}

export async function generateVisualBlocks(view, vc, lang) {
  const content = view.state.doc.toString();
  let blocks = [];
  try {
    blocks = await invoke("parse_blocks", { content, lang });
  } catch (e) {
    console.error(e);
    return;
  }

  const existing = extractMetaCoords(content);
  const toInsert = blocks.filter(
    b => !b.visual_id || !existing.has(b.visual_id)
  );
  toInsert.sort((a, b) => b.range[0] - a.range[0]);
  for (const b of toInsert) {
    const meta = insertVisualMeta(view, lang, b.range[0]);
    b.visual_id = meta.id;
  }

  if (vc && typeof vc.setBlocks === "function") {
    vc.setBlocks(blocks);
  }
  refreshBlockCount(view);
}

export async function syncNow(view, vc, lang) {
  const text = view.state.doc.toString();
  const idRegex = /\/\/\s*@VISUAL_META\s+([A-Za-z0-9_-]+)/g;
  const markers = new Set();
  let m;
  while ((m = idRegex.exec(text)) !== null) markers.add(m[1]);
  const block = ensureMetaBlock(view);
  const contentStart = text.indexOf("\n", block.start) + 1;
  const content = text.slice(contentStart, block.end - 2);
  const lines = content.split("\n");
  const newLines = [];
  const metaIds = new Set();
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      const obj = JSON.parse(trimmed);
      if (!obj.id) continue;
      if (markers.has(obj.id)) {
        if (Array.isArray(obj.links)) {
          obj.links = obj.links.filter(l => markers.has(l));
        }
        newLines.push(JSON.stringify(obj));
        metaIds.add(obj.id);
      } else {
        emit('blockRemoved', { id: obj.id });
      }
    } catch (_) {
      // ignore
    }
  }
  markers.forEach(id => {
    if (!metaIds.has(id)) {
      const obj = tmplObj();
      obj.id = id;
      newLines.push(JSON.stringify(obj));
    }
  });
  const newContent = newLines.join("\n") + "\n";
  const endPos = block.end - 2;
  view.dispatch({ changes: { from: contentStart, to: endPos, insert: newContent } });
  rebuildMetaPositions(view.state.doc.toString());
  refreshBlockCount(view);
  if (vc && typeof vc.setBlocks === 'function') {
    await generateVisualBlocks(view, vc, lang);
  }
}

export function addBlockToolbar(view, vc, lang) {
  const parent = view.dom.parentNode;
  if (!parent) return;
  const toolbar = document.createElement("div");
  toolbar.className = "editor-toolbar";
  const btn = document.createElement("button");
  btn.textContent = "Generate Blocks";
  const sync = document.createElement("button");
  sync.textContent = "Sync now";
  const countSpan = document.createElement("span");
  countSpan.id = "block-count";
  countSpan.style.marginLeft = "0.5em";
  toolbar.appendChild(btn);
  toolbar.appendChild(sync);
  toolbar.appendChild(countSpan);
  parent.insertBefore(toolbar, view.dom);

  btn.addEventListener("click", () => generateVisualBlocks(view, vc, lang));
  sync.addEventListener("click", () => syncNow(view, vc, lang));
  refreshBlockCount(view);
}

async function runTestsForBlock(id, commands) {
  if (!Array.isArray(commands) || !commands.length) return;
  try {
    await invoke("run_tests", { commands });
    if (typeof window !== "undefined") {
      emit('testResult', { id, success: true });
    }
  } catch (e) {
    if (typeof window !== "undefined") {
      emit('testResult', { id, success: false });
    }
  }
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
    const old = JSON.parse(JSON.stringify(obj));
    delete old.history;
    if (!Array.isArray(obj.history)) obj.history = [];
    obj.history.push({ timestamp: obj.updated_at || new Date().toISOString(), snapshot: old });
    if (typeof meta.x === "number") obj.x = meta.x;
    if (typeof meta.y === "number") obj.y = meta.y;
    if (Array.isArray(meta.tags)) {
      obj.tags = meta.tags;
    }
    if (Array.isArray(meta.links)) {
      obj.links = meta.links;
    }
    if (Array.isArray(meta.tests)) {
      obj.tests = meta.tests;
    }
    obj.updated_at = new Date().toISOString();
    const newJson = JSON.stringify(obj);
    view.dispatch({ changes: { from: pos, to: pos + json.length, insert: newJson } });
    rebuildMetaPositions(view.state.doc.toString());
    if (Array.isArray(obj.tests) && obj.tests.length) {
      runTestsForBlock(obj.id, obj.tests);
    }
    return true;
  } catch (_) {
    return false;
  }
}

export function getMetaById(view, id) {
  let pos = metaPositions.get(id);
  if (pos == null) {
    rebuildMetaPositions(view.state.doc.toString());
    pos = metaPositions.get(id);
  }
  if (pos == null) return null;
  const text = view.state.doc.toString();
  const end = text.indexOf("\n", pos);
  const json = text.slice(pos, end === -1 ? undefined : end);
  try {
    return JSON.parse(json);
  } catch (_) {
    return null;
  }
}

export function previewDiff(patch) {
  const parsed = parsePatch(patch);
  const overlay = document.createElement('div');
  overlay.style.position = 'fixed';
  overlay.style.top = '0';
  overlay.style.left = '0';
  overlay.style.right = '0';
  overlay.style.bottom = '0';
  overlay.style.background = 'rgba(0,0,0,0.4)';
  overlay.style.display = 'flex';
  overlay.style.alignItems = 'center';
  overlay.style.justifyContent = 'center';
  overlay.style.zIndex = '10000';

  const box = document.createElement('div');
  box.style.background = '#fff';
  box.style.padding = '1em';
  box.style.maxHeight = '80%';
  box.style.maxWidth = '80%';
  box.style.overflow = 'auto';

  const pre = document.createElement('pre');
  parsed.forEach(p => {
    p.hunks.forEach(h => {
      h.lines.forEach(line => {
        const span = document.createElement('span');
        if (line.startsWith('+')) {
          span.style.background = '#dfd';
        } else if (line.startsWith('-')) {
          span.style.background = '#fdd';
        }
        span.textContent = line + '\n';
        pre.appendChild(span);
      });
    });
  });
  box.appendChild(pre);

  const btns = document.createElement('div');
  btns.style.textAlign = 'right';
  btns.style.marginTop = '0.5em';
  const apply = document.createElement('button');
  apply.textContent = 'Apply';
  const cancel = document.createElement('button');
  cancel.textContent = 'Cancel';
  cancel.style.marginLeft = '0.5em';
  btns.appendChild(apply);
  btns.appendChild(cancel);
  box.appendChild(btns);

  overlay.appendChild(box);
  document.body.appendChild(overlay);

  return new Promise(resolve => {
    apply.onclick = () => {
      document.body.removeChild(overlay);
      resolve(true);
    };
    cancel.onclick = () => {
      document.body.removeChild(overlay);
      resolve(false);
    };
  });
}

async function confirmRename(matches, oldId, newId) {
  const overlay = document.createElement('div');
  overlay.style.position = 'fixed';
  overlay.style.top = '0';
  overlay.style.left = '0';
  overlay.style.right = '0';
  overlay.style.bottom = '0';
  overlay.style.background = 'rgba(0,0,0,0.4)';
  overlay.style.display = 'flex';
  overlay.style.alignItems = 'center';
  overlay.style.justifyContent = 'center';
  overlay.style.zIndex = '10000';

  const box = document.createElement('div');
  box.style.background = '#fff';
  box.style.padding = '1em';
  box.style.maxHeight = '80%';
  box.style.maxWidth = '80%';
  box.style.overflow = 'auto';

  const title = document.createElement('div');
  title.textContent = `Rename "${oldId}" to "${newId}" in:`;
  box.appendChild(title);

  const list = document.createElement('pre');
  matches.forEach(m => {
    list.appendChild(document.createTextNode(`line ${m.line}: ${m.text.trim()}\n`));
  });
  box.appendChild(list);

  const btns = document.createElement('div');
  btns.style.textAlign = 'right';
  btns.style.marginTop = '0.5em';
  const apply = document.createElement('button');
  apply.textContent = 'Rename';
  const cancel = document.createElement('button');
  cancel.textContent = 'Cancel';
  cancel.style.marginLeft = '0.5em';
  btns.appendChild(apply);
  btns.appendChild(cancel);
  box.appendChild(btns);

  overlay.appendChild(box);
  document.body.appendChild(overlay);

  return new Promise(resolve => {
    apply.onclick = () => {
      document.body.removeChild(overlay);
      resolve(true);
    };
    cancel.onclick = () => {
      document.body.removeChild(overlay);
      resolve(false);
    };
  });
}

function replaceInView(view, from, to, text) {
  if (view && typeof view.replace === 'function') {
    view.replace(from, to, text);
  } else {
    view.dispatch({ changes: { from, to, insert: text } });
  }
}

export async function renameMetaId(view, oldId, newId) {
  if (!view || !oldId || !newId) return false;
  const text = view.state.doc.toString();
  const esc = oldId.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const re = new RegExp(esc, 'g');
  const matches = [];
  let m;
  while ((m = re.exec(text)) !== null) {
    const from = m.index;
    const to = from + oldId.length;
    const lineInfo = view.state.doc.lineAt(from);
    matches.push({ from, to, line: lineInfo.number, text: lineInfo.text });
  }
  if (!matches.length) return false;
  const ok = await confirmRename(matches, oldId, newId);
  if (!ok) return false;
  // apply replacements from last to first to keep positions
  for (let i = matches.length - 1; i >= 0; i--) {
    const m = matches[i];
    replaceInView(view, m.from, m.to, newId);
  }
  rebuildMetaPositions(view.state.doc.toString());
  return true;
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

function findInlineMetaPositions(text) {
  const positions = new Map();
  for (const re of regexes) {
    re.lastIndex = 0;
    let m;
    while ((m = re.exec(text)) !== null) {
      try {
        const obj = JSON.parse(m[1]);
        if (obj && obj.id) {
          positions.set(obj.id, { start: m.index, end: m.index + m[0].length });
        }
      } catch (_) {
        /* ignore */
      }
    }
  }
  return positions;
}

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
    this.fileId = view.dom.dataset.fileId || 'current';
    currentView = view;
    this.onLinterMessage = this.onLinterMessage.bind(this);
    this.onClick = this.onClick.bind(this);
    this.onMouseMove = this.onMouseMove.bind(this);
    this.onMouseLeave = this.onMouseLeave.bind(this);
    this.lastHoverId = null;
    this.lastMouse = { x: 0, y: 0 };
    this.prevCoords = extractMetaCoords(view.state.doc.toString());
    this.knownIds = new Set();
    const idRegex = /\/\/\s*@VISUAL_META\s+([A-Za-z0-9_-]+)/g;
    let m;
    const text = view.state.doc.toString();
    while ((m = idRegex.exec(text)) !== null) {
      this.knownIds.add(m[1]);
    }
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
    this.unsub = [];
    this.unsub.push(
      on('blockInfo', ({ id, kind, color, thumbnail }) => {
        if (id === this.lastHoverId) {
          if (thumbnail) this.tooltip.innerHTML = `<img src="${thumbnail}" />`;
          else this.tooltip.textContent = `${kind || ''} ${color || ''}`.trim();
          this.tooltip.style.left = this.lastMouse.x + 10 + 'px';
          this.tooltip.style.top = this.lastMouse.y + 10 + 'px';
          this.tooltip.style.display = 'block';
        }
      }),
      on('blocksReordered', ({ ids }) => {
        if (Array.isArray(ids) && settings?.visual?.syncOrder !== false) reorderMeta(ids);
      }),
      on('edgeSelected', ({ from, to }) => {
        highlightRangeBetweenIds(this.view, from, to);
      }),
      on('refreshText', ({ updates }) => {
        const txt = updates[this.fileId];
        if (typeof txt === 'string') {
          const doc = this.view.state.doc.toString();
          this.view.dispatch({ changes: { from: 0, to: doc.length, insert: txt } });
          rebuildMetaPositions(this.view.state.doc.toString());
        }
      }),
      on('blockSelected', ({ id }) => {
        highlightMetaById(this.view, id);
      })
    );
    window.addEventListener('message', this.onLinterMessage);
    view.dom.addEventListener('click', this.onClick);
    view.dom.addEventListener('mousemove', this.onMouseMove);
    view.dom.addEventListener('mouseleave', this.onMouseLeave);
    this.consistencyErrors = new Map();
    this.postLint = this.postLint.bind(this);
    this.checkConsistency = this.checkConsistency.bind(this);
    this.checkInterval = setInterval(this.checkConsistency, 5000);
    this.checkConsistency();
  }
  destroy() {
    window.removeEventListener('message', this.onLinterMessage);
    this.unsub.forEach(u => u());
    this.view.dom.removeEventListener('click', this.onClick);
    this.view.dom.removeEventListener('mousemove', this.onMouseMove);
    this.view.dom.removeEventListener('mouseleave', this.onMouseLeave);
    document.body.removeChild(this.tooltip);
    clearTimeout(this.debounceTimer);
    clearInterval(this.checkInterval);
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
          emit('metaUpdated', { id, x: meta.x, y: meta.y });
        }
      }
    }
    this.prevCoords = current;
  }
  processDirectives() {
    const text = this.view.state.doc.toString();
    const blockRegex = /\/\/\s*@VISUAL_BLOCK\s+([A-Za-z0-9_-]+)/g;
    let m;
    const matches = [];
    while ((m = blockRegex.exec(text)) !== null) {
      matches.push({ kind: m[1], start: m.index, end: m.index + m[0].length });
    }
    for (const match of matches.reverse()) {
      const id = crypto.randomUUID();
      const marker = `// @VISUAL_META ${id}`;
      this.view.dispatch({ changes: { from: match.start, to: match.end, insert: marker } });
      const block = ensureMetaBlock(this.view);
      const insertPos = block.end - 2;
      const metaObj = { id, x: 0, y: 0, updated_at: new Date().toISOString() };
      const jsonLine = `${JSON.stringify(metaObj)}\n`;
      this.view.dispatch({ changes: { from: insertPos, to: insertPos, insert: jsonLine } });
      this.knownIds.add(id);
      emit('blockCreated', { id, kind: match.kind });
    }
  }
  detectRemoved() {
    const text = this.view.state.doc.toString();
    const idRegex = /\/\/\s*@VISUAL_META\s+([A-Za-z0-9_-]+)/g;
    const current = new Set();
    let m;
    while ((m = idRegex.exec(text)) !== null) current.add(m[1]);
    for (const id of Array.from(this.knownIds)) {
      if (!current.has(id)) {
        emit('blockRemoved', { id });
      }
    }
    this.knownIds = current;
  }
  update(update) {
    if (update.docChanged) {
      this.processDirectives();
      this.detectRemoved();
      this.scheduleSync();
    }
  }
  onLinterMessage(e) {
    const { source, diagnostics } = e.data || {};
    if (source === 'linter' && Array.isArray(diagnostics)) {
      lintMessages.clear();
      const text = this.view.state.doc.toString();
      for (const d of diagnostics) {
        const p = typeof d.from === 'number' ? d.from : (typeof d.to === 'number' ? d.to : 0);
        const bid = findIdForPos(text, p);
        if (bid) {
          const msg = d.message || d.msg || '';
          if (lintMessages.has(bid)) lintMessages.set(bid, lintMessages.get(bid) + '\n' + msg);
          else lintMessages.set(bid, msg);
        }
      }
      this.postLint();
    }
  }
  checkConsistency() {
    const text = this.view.state.doc.toString();
    const idRegex = /\/\/\s*@VISUAL_META\s+([A-Za-z0-9_-]+)/g;
    const markers = new Set();
    let m;
    while ((m = idRegex.exec(text)) !== null) markers.add(m[1]);
    const block = getMetaBlock(text);
    const errors = new Map();
    const metaIds = new Set();
    if (block) {
      const contentStart = text.indexOf('\n', block.start) + 1;
      const content = text.slice(contentStart, block.end - 2);
      const lines = content.split('\n');
      for (const line of lines) {
        const trimmed = line.trim();
        if (!trimmed) continue;
        try {
          const obj = JSON.parse(trimmed);
          if (!obj.id) continue;
          metaIds.add(obj.id);
          if (typeof obj.x !== 'number' || typeof obj.y !== 'number') {
            errors.set(obj.id, 'Invalid position');
          }
        } catch (_) {
          // ignore
        }
      }
    }
    markers.forEach(id => {
      if (!metaIds.has(id)) errors.set(id, 'Missing meta entry');
    });
    metaIds.forEach(id => {
      if (!markers.has(id)) errors.set(id, 'Orphan meta entry');
    });
    this.consistencyErrors = errors;
    this.postLint();
  }
  postLint() {
    const merged = new Map([...lintMessages, ...this.consistencyErrors]);
    editorDiagnostics(this.view, merged);
    emit('lintReported', { errors: Object.fromEntries(merged) });
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
      emit('blockSelected', { id: clickedId });
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
        emit('blockInfoRequest', { id: hoverId });
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


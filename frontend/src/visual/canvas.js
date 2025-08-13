import { createBlock } from './blocks.js';
import { getTheme, onThemeChange } from './theme.ts';
import { registerHoverHighlight, drawHoverHighlight } from './hover.ts';
import { Minimap } from './minimap.ts';
import settings from '../../settings.json' assert { type: 'json' };
import { createTwoFilesPatch } from 'diff';
import { updateMetaComment, previewDiff } from '../editor/visual-meta.js';

export const VIEW_STATE_KEY = 'visual-view-state';

const cfg = settings.visual || {};
const GRID_SIZE = cfg.gridSize || 20;
const MIN_SCALE = 0.5;
const MAX_SCALE = 4;

// Utility used in tests and debug mode to analyze graph connections.
// Accepts an array of block ids and array of edges [fromId, toId].
// Returns sets of block ids that lack connections and edges participating in cycles.
export function analyzeConnections(blockIds, edges) {
  const missing = new Set();
  const cycles = new Set();

  const adjacency = new Map();
  const connected = new Set();
  for (const [a, b] of edges) {
    connected.add(a);
    connected.add(b);
    if (!adjacency.has(a)) adjacency.set(a, []);
    adjacency.get(a).push(b);
  }

  blockIds.forEach(id => {
    if (!connected.has(id)) missing.add(id);
  });

  const visited = new Set();
  const stack = [];
  const onStack = new Set();

  function dfs(node) {
    stack.push(node);
    onStack.add(node);
    const neigh = adjacency.get(node) || [];
    for (const n of neigh) {
      if (onStack.has(n)) {
        const idx = stack.indexOf(n);
        for (let i = idx; i < stack.length - 1; i++) {
          cycles.add(stack[i] + '->' + stack[i + 1]);
        }
        cycles.add(stack[stack.length - 1] + '->' + n);
      } else if (!visited.has(n)) {
        dfs(n);
      }
    }
    stack.pop();
    onStack.delete(node);
    visited.add(node);
  }

  adjacency.forEach((_, id) => {
    if (!visited.has(id)) dfs(id);
  });

  return { missing, cycles };
}

export class VisualCanvas {
  constructor(canvas, minimapCanvas = null) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.minimap = minimapCanvas ? new Minimap(minimapCanvas, this) : null;
    this.scale = 1;
    this.offset = { x: 0, y: 0 };
    this.blocks = [];
    this.blocksData = [];
    this.blockDataMap = new Map();
    this.locale = 'en';
    this.connections = [];
    this.debugMode = false;
    this.errorBlocks = new Map();
    this.errorEdges = new Map();
    this.dragged = null;
    this.dragOffset = { x: 0, y: 0 };
    this.draggingConnection = null;
    this.panning = false;
    this.panStart = { x: 0, y: 0 };
    this.moveCallback = null;
    this.metaView = null;
    this.undoStack = [];
    this.redoStack = [];
    this.dragStart = { x: 0, y: 0 };
    this.highlighted = new Set();
    this.hovered = null;
    this.gridEnabled = cfg.showGrid ?? false;
    this.selected = new Set();
    this.groups = new Map();
    this.nextGroupId = 1;
    this.alignGuides = [];
    this.selectionBox = null;
    this.missingEdge = null;
    this.nextAutoPos = { x: 0, y: 0 };

    this.tooltip = document.createElement('div');
    const theme = getTheme();
    this.tooltip.style.position = 'fixed';
    this.tooltip.style.background = theme.tooltipBg;
    this.tooltip.style.color = theme.tooltipText;
    this.tooltip.style.padding = '4px 8px';
    this.tooltip.style.borderRadius = '4px';
    this.tooltip.style.pointerEvents = 'none';
    this.tooltip.style.whiteSpace = 'pre';
    this.tooltip.style.display = 'none';
    document.body.appendChild(this.tooltip);

    onThemeChange(t => {
      this.tooltip.style.background = t.tooltipBg;
      this.tooltip.style.color = t.tooltipText;
    });

    this.resize();
    window.addEventListener('resize', () => this.resize());
    this.registerEvents();
    registerHoverHighlight(this);
    window.addEventListener('message', e => {
      const { source, id, type, x, y, from, to, kind } = e.data || {};
      if (source === 'visual-meta') {
        if (type === 'request-block-info' && id) {
          const data = this.blockDataMap.get(id);
          if (data) {
            const theme = getTheme();
            const color = theme.blockKinds[data.kind] || theme.blockFill;
            let thumbnail = null;
            try {
              const block = createBlock(data.kind, id, 0, 0, data.kind, color);
              const off = document.createElement('canvas');
              off.width = block.w;
              off.height = block.h;
              const ctx = off.getContext('2d');
              block.draw(ctx);
              thumbnail = off.toDataURL();
            } catch (_) {
              thumbnail = null;
            }
            window.postMessage({ source: 'visual-canvas', type: 'block-info', id, kind: data.kind, color, thumbnail }, '*');
          }
        } else if (type === 'updatePos' && id) {
          const data = this.blockDataMap.get(id);
          if (data) {
            if (typeof x === 'number') data.x = x;
            if (typeof y === 'number') data.y = y;
            this.updateLabels();
            this.draw();
          }
        } else if (type === 'edge-not-found' && from && to) {
          this.missingEdge = from + '->' + to;
          setTimeout(() => {
            if (this.missingEdge === from + '->' + to) this.missingEdge = null;
          }, 2000);
        } else if (type === 'create-block' && id && kind) {
          const pos = this.getFreePos();
          const theme = getTheme();
          const color = theme.blockKinds[kind] || theme.blockFill;
          const block = createBlock(kind, id, pos.x, pos.y, kind, color);
          this.blocks.push(block);
          const data = { visual_id: id, kind, x: pos.x, y: pos.y, tags: [], links: [], updated_at: new Date().toISOString() };
          this.blocksData.push(data);
          this.blockDataMap.set(id, data);
          if (this.metaView) updateMetaComment(this.metaView, { id, x: pos.x, y: pos.y });
          this.draw();
        } else if (type === 'remove-block' && id) {
          this.blocks = this.blocks.filter(b => b.id !== id);
          this.blocksData = this.blocksData.filter(b => b.visual_id !== id);
          this.blockDataMap.delete(id);
          this.connections = this.connections.filter(([a, b]) => a.id !== id && b.id !== id);
          this.draw();
        } else if (id) {
          this.highlightBlocks([id]);
        }
      }
    });
    requestAnimationFrame(() => this.draw());
  }

  setBlocks(blocks) {
    this.blocksData = blocks;
    this.blockDataMap = new Map(blocks.map(b => [b.visual_id, b]));
    this.updateLabels();
    this.highlightBlocks([]);
    this.connections = [];
    const byId = new Map(this.blocks.map(b => [b.id, b]));
    blocks.forEach(b => {
      if (Array.isArray(b.links)) {
        b.links.forEach(l => {
          const from = byId.get(b.visual_id);
          const to = byId.get(l);
          if (from && to) this.connections.push([from, to]);
        });
      }
    });
    this.analyze();
  }

  setLocale(locale) {
    this.locale = locale;
    this.updateLabels();
  }

  updateLabels() {
    const theme = getTheme();
    this.blocks = this.blocksData.map(b => {
      const label = (b.translations && b.translations[this.locale]) || b.kind;
      const base = theme.blockKinds[b.kind] || theme.blockFill;
      const color = this.highlighted.has(b.visual_id) ? theme.highlight : base;
      return createBlock(b.kind, b.visual_id, b.x, b.y, label, color);
    });
  }

  highlightBlocks(ids) {
    const theme = getTheme();
    this.highlighted = new Set(ids);
    this.blocks.forEach(b => {
      const data = this.blockDataMap.get(b.id);
      const base = data ? theme.blockKinds[data.kind] || theme.blockFill : theme.blockFill;
      b.color = this.highlighted.has(b.id) ? theme.highlight : base;
    });
  }

  selectBlock(id) {
    if (id) this.highlightBlocks([id]);
    else this.highlightBlocks([]);
    window.postMessage({ source: 'visual-canvas', id }, '*');
  }

  search(label) {
    const query = (label || '').trim().toLowerCase();
    if (!query) {
      this.highlightBlocks([]);
      return;
    }
    const ids = this.blocks
      .filter(b => b.label.toLowerCase().includes(query))
      .map(b => b.id);
    this.highlightBlocks(ids);
  }

  getGroupId(blockId) {
    for (const [id, set] of this.groups.entries()) {
      if (set.has(blockId)) return id;
    }
    return null;
  }

  groupSelected() {
    if (this.selected.size === 0) return;
    const id = this.nextGroupId++;
    this.groups.set(id, new Set(Array.from(this.selected).map(b => b.id)));
  }

  ungroupSelected() {
    const ids = new Set(Array.from(this.selected).map(b => b.id));
    for (const [id, set] of Array.from(this.groups.entries())) {
      for (const bid of ids) {
        if (set.has(bid)) {
          this.groups.delete(id);
          break;
        }
      }
    }
  }

  onBlockMove(cb) {
    this.moveCallback = cb;
  }

  setMetaView(view) {
    this.metaView = view;
  }

  getFreePos() {
    const pos = { ...this.nextAutoPos };
    this.nextAutoPos.x += 150;
    if (this.nextAutoPos.x > 1000) {
      this.nextAutoPos.x = 0;
      this.nextAutoPos.y += 150;
    }
    return pos;
  }

  addBlock(block) {
    this.blocks.push(block);
  }

  serialize() {
    return {
      blocks: this.blocksData,
      connections: this.connections.map(([a, b]) => [a.id, b.id]),
      offset: this.offset,
      scale: this.scale
    };
  }

  load(layout) {
    if (!layout) return;
    this.blocksData = layout.blocks || [];
    this.blockDataMap = new Map(this.blocksData.map(b => [b.visual_id, b]));
    this.updateLabels();
    const byId = new Map(this.blocks.map(b => [b.id, b]));
    this.connections = (layout.connections || [])
      .map(([a, b]) => {
        const from = byId.get(a);
        const to = byId.get(b);
        return from && to ? [from, to] : null;
      })
      .filter(Boolean);
    this.offset = layout.offset || { x: 0, y: 0 };
    this.scale = layout.scale ?? 1;
    this.analyze();
  }

  saveViewState() {
    if (typeof localStorage === 'undefined') return;
    try {
      localStorage.setItem(
        VIEW_STATE_KEY,
        JSON.stringify({ offset: this.offset, scale: this.scale })
      );
    } catch (_err) {
      // ignore storage errors
    }
  }

  connect(a, b) {
    this.connections.push([a, b]);
    this.analyze();
  }

  setDebugMode(enabled) {
    this.debugMode = enabled;
    this.analyze();
  }

  setGridEnabled(enabled) {
    this.gridEnabled = enabled;
  }

  analyze() {
    const ids = this.blocks.map(b => b.id);
    const edges = this.connections.map(([a, b]) => [a.id, b.id]);
    const { missing, cycles } = analyzeConnections(ids, edges);
    this.errorBlocks = new Map();
    this.errorEdges = new Map();
    missing.forEach(id => this.errorBlocks.set(id, 'Missing connection'));
    cycles.forEach(edge => {
      this.errorEdges.set(edge, 'Cyclic connection');
      const [from, to] = edge.split('->');
      this.errorBlocks.set(from, 'Cyclic connection');
      this.errorBlocks.set(to, 'Cyclic connection');
    });
  }

  registerEvents() {
    this.canvas.addEventListener('mousedown', e => {
      const pos = this.toWorld(e.offsetX, e.offsetY);
      const block = this.blocks.find(b => b.contains(pos.x, pos.y));

      let edge = null;
      if (!block) {
        edge = this.connections.find(([a, b]) => this.pointToSegmentDist(pos, a.center(), b.center()) < 5);
      }

      if (edge) {
        this.selectBlock(null);
        window.postMessage({ source: 'visual-canvas', type: 'edgeSelected', from: edge[0].id, to: edge[1].id }, '*');
        this.dragged = null;
        this.panning = false;
        this.tooltip.style.display = 'none';
        return;
      }

      if (block) this.selectBlock(block.id); else this.selectBlock(null);

      if (block) {
        const exit = { x: block.x + block.w, y: block.y + block.h / 2 };
        if (Math.hypot(pos.x - exit.x, pos.y - exit.y) < 5) {
          this.draggingConnection = { from: block, x: pos.x, y: pos.y };
          this.dragged = null;
          this.panning = false;
          this.tooltip.style.display = 'none';
          return;
        }
      }

      if (e.shiftKey) {
        if (block) {
          if (this.selected.has(block)) this.selected.delete(block);
          else this.selected.add(block);
        } else {
          this.selectionBox = { startX: pos.x, startY: pos.y, x: pos.x, y: pos.y };
        }
        this.dragged = null;
        this.panning = false;
      } else {
        this.dragged = block;
        if (this.dragged) {
          this.dragOffset.x = pos.x - this.dragged.x;
          this.dragOffset.y = pos.y - this.dragged.y;
          this.dragStart.x = this.dragged.x;
          this.dragStart.y = this.dragged.y;
        } else {
          this.panning = true;
          this.panStart.x = e.offsetX - this.offset.x;
          this.panStart.y = e.offsetY - this.offset.y;
        }
      }
      this.tooltip.style.display = 'none';
    });

    this.canvas.addEventListener('dblclick', e => {
      const pos = this.toWorld(e.offsetX, e.offsetY);
      const block = this.blocks.find(b => b.contains(pos.x, pos.y));
      if (block && typeof window.openInTextEditor === 'function') {
        this.saveViewState();
        window.openInTextEditor(block.id);
      }
    });

    this.canvas.addEventListener('mousemove', e => {
      const pos = this.toWorld(e.offsetX, e.offsetY);
      if (this.selectionBox) {
        this.selectionBox.x = pos.x;
        this.selectionBox.y = pos.y;
      } else if (this.draggingConnection) {
        this.draggingConnection.x = pos.x;
        this.draggingConnection.y = pos.y;
      } else if (this.dragged) {
        let x = pos.x - this.dragOffset.x;
        let y = pos.y - this.dragOffset.y;
        if (this.gridEnabled) {
          x = Math.round(x / GRID_SIZE) * GRID_SIZE;
          y = Math.round(y / GRID_SIZE) * GRID_SIZE;
        }
        const oldX = this.dragged.x;
        const oldY = this.dragged.y;
        this.dragged.x = x;
        this.dragged.y = y;
        const dx = this.dragged.x - oldX;
        const dy = this.dragged.y - oldY;
        const gid = this.getGroupId(this.dragged.id);
        if (gid !== null) {
          const set = this.groups.get(gid);
          for (const id of set) {
            if (id === this.dragged.id) continue;
            const b = this.blocks.find(bb => bb.id === id);
            if (!b) continue;
            let bx = b.x + dx;
            let by = b.y + dy;
            if (this.gridEnabled) {
              bx = Math.round(bx / GRID_SIZE) * GRID_SIZE;
              by = Math.round(by / GRID_SIZE) * GRID_SIZE;
            }
            b.x = bx;
            b.y = by;
          }
        } else if (this.selected.has(this.dragged)) {
          for (const b of this.selected) {
            if (b === this.dragged) continue;
            let bx = b.x + dx;
            let by = b.y + dy;
            if (this.gridEnabled) {
              bx = Math.round(bx / GRID_SIZE) * GRID_SIZE;
              by = Math.round(by / GRID_SIZE) * GRID_SIZE;
            }
            b.x = bx;
            b.y = by;
          }
        }
        this.updateAlignGuides();
      } else {
        this.alignGuides = [];
        if (this.panning) {
        this.offset.x = e.offsetX - this.panStart.x;
        this.offset.y = e.offsetY - this.panStart.y;
        } else {
          const hovered = this.blocks.find(b => b.contains(pos.x, pos.y));
          let tooltipText = null;
          if (hovered) {
            const data = this.blockDataMap.get(hovered.id);
            const note = data && data.ai;
            if (note && (note.description || (note.hints && note.hints.length))) {
              const lines = [];
              if (note.description) lines.push(note.description);
              if (note.hints) lines.push(...note.hints);
              tooltipText = lines.join('\n');
            }
            const err = this.errorBlocks.get(hovered.id);
            if (err) tooltipText = err;
          } else {
            for (const [edge, msg] of this.errorEdges.entries()) {
              const [fromId, toId] = edge.split('->');
              const a = this.blocks.find(b => b.id === fromId);
              const b = this.blocks.find(b => b.id === toId);
              if (a && b && this.pointToSegmentDist(pos, a.center(), b.center()) < 5) {
                tooltipText = msg;
                break;
              }
            }
          }

          if (tooltipText) {
            this.tooltip.textContent = tooltipText;
            const rect = this.canvas.getBoundingClientRect();
            this.tooltip.style.left = rect.left + e.offsetX + 10 + 'px';
            this.tooltip.style.top = rect.top + e.offsetY + 10 + 'px';
            this.tooltip.style.display = 'block';
          } else {
            this.tooltip.style.display = 'none';
          }
        }
      }
    });

    window.addEventListener('mouseup', async e => {
      const wasDragged = !!this.dragged;
      let appliedMove = false;
      if (this.selectionBox) {
        const { startX, startY, x, y } = this.selectionBox;
        const x1 = Math.min(startX, x);
        const y1 = Math.min(startY, y);
        const x2 = Math.max(startX, x);
        const y2 = Math.max(startY, y);
        const sel = new Set();
        for (const b of this.blocks) {
          if (b.x >= x1 && b.x + b.w <= x2 && b.y >= y1 && b.y + b.h <= y2) sel.add(b);
        }
        this.selected = sel;
        this.selectionBox = null;
      }
      if (this.draggingConnection) {
        const rect = this.canvas.getBoundingClientRect();
        const offX = e.clientX - rect.left;
        const offY = e.clientY - rect.top;
        const pos = this.toWorld(offX, offY);
        const target = this.blocks.find(b => b.contains(pos.x, pos.y));
        if (target && target !== this.draggingConnection.from) {
          this.connections.push([this.draggingConnection.from, target]);
          this.undoStack.push({
            type: 'connect',
            from: this.draggingConnection.from.id,
            to: target.id
          });
          this.redoStack = [];
          this.analyze();
        }
        this.draggingConnection = null;
      }
      if (this.dragged) {
        if (this.gridEnabled) {
          const snap = b => {
            b.x = Math.round(b.x / GRID_SIZE) * GRID_SIZE;
            b.y = Math.round(b.y / GRID_SIZE) * GRID_SIZE;
          };
          const targets = [this.dragged];
          const gid = this.getGroupId(this.dragged.id);
          if (gid !== null) {
            const set = this.groups.get(gid);
            for (const id of set) {
              if (id === this.dragged.id) continue;
              const b = this.blocks.find(bb => bb.id === id);
              if (b) targets.push(b);
            }
          } else if (this.selected.has(this.dragged)) {
            for (const b of this.selected) {
              if (b === this.dragged) continue;
              targets.push(b);
            }
          }
          targets.forEach(snap);
        }
        const moved = this.dragged.x !== this.dragStart.x || this.dragged.y !== this.dragStart.y;
        if (moved && this.metaView) {
          const data = this.blockDataMap.get(this.dragged.id) || { id: this.dragged.id };
          const oldObj = { ...data, x: this.dragStart.x, y: this.dragStart.y };
          const newObj = { ...data, x: this.dragged.x, y: this.dragged.y };
          const patch = createTwoFilesPatch('meta', 'meta', JSON.stringify(oldObj) + '\n', JSON.stringify(newObj) + '\n');
          const ok = await previewDiff(patch);
          if (ok) {
            this.undoStack.push({
              type: 'move',
              id: this.dragged.id,
              from: { x: this.dragStart.x, y: this.dragStart.y },
              to: { x: this.dragged.x, y: this.dragged.y }
            });
            this.redoStack = [];
            updateMetaComment(this.metaView, { id: this.dragged.id, x: this.dragged.x, y: this.dragged.y });
            if (this.moveCallback) {
              this.moveCallback(this.dragged);
            }
            appliedMove = true;
          } else {
            this.dragged.x = this.dragStart.x;
            this.dragged.y = this.dragStart.y;
            this.draw();
          }
        } else {
          if (moved) {
            this.undoStack.push({
              type: 'move',
              id: this.dragged.id,
              from: { x: this.dragStart.x, y: this.dragStart.y },
              to: { x: this.dragged.x, y: this.dragged.y }
            });
            this.redoStack = [];
          }
          if (this.moveCallback) {
            this.moveCallback(this.dragged);
          }
          appliedMove = moved;
        }
      }
      if (cfg.syncOrder && wasDragged && appliedMove) {
        const ids = this.blocks
          .slice()
          .sort((a, b) => a.y - b.y)
          .map(b => b.id);
        window.postMessage({ source: 'visual-canvas', type: 'reorder', ids }, '*');
      }
      this.dragged = null;
      this.panning = false;
      this.alignGuides = [];
    });

    this.canvas.addEventListener('mouseleave', () => {
      this.tooltip.style.display = 'none';
    });

    this.canvas.addEventListener('wheel', e => {
      e.preventDefault();
      const mouseX = e.offsetX;
      const mouseY = e.offsetY;
      const worldPos = this.toWorld(mouseX, mouseY);
      const scaleFactor = e.deltaY < 0 ? 1.1 : 0.9;
      this.scale *= scaleFactor;
      this.scale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, this.scale));
      const newScreenX = worldPos.x * this.scale + this.offset.x;
      const newScreenY = worldPos.y * this.scale + this.offset.y;
      this.offset.x += mouseX - newScreenX;
      this.offset.y += mouseY - newScreenY;
    });
  }

  resize() {
    this.canvas.width = this.canvas.clientWidth;
    this.canvas.height = this.canvas.clientHeight;
  }

  zoomToFit() {
    if (this.blocks.length === 0) return;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const b of this.blocks) {
      minX = Math.min(minX, b.x);
      minY = Math.min(minY, b.y);
      maxX = Math.max(maxX, b.x + b.w);
      maxY = Math.max(maxY, b.y + b.h);
    }
    const width = maxX - minX;
    const height = maxY - minY;
    if (width === 0 || height === 0) return;
    const scale = Math.min(this.canvas.width / width, this.canvas.height / height) * 0.9;
    this.scale = Math.max(MIN_SCALE, Math.min(MAX_SCALE, scale));
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    this.offset.x = this.canvas.width / 2 - cx * this.scale;
    this.offset.y = this.canvas.height / 2 - cy * this.scale;
  }

  toWorld(x, y) {
    return {
      x: (x - this.offset.x) / this.scale,
      y: (y - this.offset.y) / this.scale
    };
  }

  pointToSegmentDist(p, a, b) {
    const A = p.x - a.x;
    const B = p.y - a.y;
    const C = b.x - a.x;
    const D = b.y - a.y;
    const dot = A * C + B * D;
    const lenSq = C * C + D * D;
    let param = -1;
    if (lenSq !== 0) param = dot / lenSq;
    let xx, yy;
    if (param < 0) { xx = a.x; yy = a.y; }
    else if (param > 1) { xx = b.x; yy = b.y; }
    else { xx = a.x + param * C; yy = a.y + param * D; }
    const dx = p.x - xx;
    const dy = p.y - yy;
    return Math.sqrt(dx * dx + dy * dy);
  }

  updateAlignGuides() {
    if (!this.dragged) {
      this.alignGuides = [];
      return;
    }

    const threshold = 5;
    const d = this.dragged;

    // Potential x and y lines for the dragged block (left, center, right / top, middle, bottom)
    const dxs = [d.x, d.x + d.w / 2, d.x + d.w];
    const dys = [d.y, d.y + d.h / 2, d.y + d.h];

    const vSet = new Set();
    const hSet = new Set();

    for (const b of this.blocks) {
      if (b === d) continue;
      const bx = [b.x, b.x + b.w / 2, b.x + b.w];
      const by = [b.y, b.y + b.h / 2, b.y + b.h];

      for (const x1 of dxs) {
        for (const x2 of bx) {
          if (Math.abs(x1 - x2) < threshold) vSet.add(x2);
        }
      }

      for (const y1 of dys) {
        for (const y2 of by) {
          if (Math.abs(y1 - y2) < threshold) hSet.add(y2);
        }
      }
    }

    const lines = [
      ...Array.from(vSet, x => ({ type: 'v', x })),
      ...Array.from(hSet, y => ({ type: 'h', y })),
    ];
    this.alignGuides = lines;
  }

  draw() {
    const theme = getTheme();
    this.ctx.save();
    this.ctx.setTransform(this.scale, 0, 0, this.scale, this.offset.x, this.offset.y);
    this.ctx.clearRect(-this.offset.x / this.scale, -this.offset.y / this.scale,
      this.canvas.width / this.scale, this.canvas.height / this.scale);

    if (this.gridEnabled) {
      const size = GRID_SIZE;
      const width = this.canvas.width / this.scale;
      const height = this.canvas.height / this.scale;
      const startX = Math.floor((-this.offset.x / this.scale) / size) * size;
      const startY = Math.floor((-this.offset.y / this.scale) / size) * size;
      const endX = startX + width + size;
      const endY = startY + height + size;
      this.ctx.beginPath();
      this.ctx.strokeStyle = '#eee';
      this.ctx.lineWidth = 1 / this.scale;
      for (let x = startX; x <= endX; x += size) {
        this.ctx.moveTo(x, startY);
        this.ctx.lineTo(x, endY);
      }
      for (let y = startY; y <= endY; y += size) {
        this.ctx.moveTo(startX, y);
        this.ctx.lineTo(endX, y);
      }
      this.ctx.stroke();
    }

    // Draw connections
    this.connections.forEach(([a, b]) => {
      const ac = a.center();
      const bc = b.center();
      const key = a.id + '->' + b.id;
      this.ctx.beginPath();
      this.ctx.moveTo(ac.x, ac.y);
      this.ctx.lineTo(bc.x, bc.y);
      if (this.errorEdges.has(key)) {
        this.ctx.strokeStyle = 'red';
        this.ctx.lineWidth = 2;
        if (this.ctx.setLineDash) this.ctx.setLineDash([]);
      } else if (this.missingEdge === key) {
        this.ctx.strokeStyle = 'orange';
        this.ctx.lineWidth = 2;
        if (this.ctx.setLineDash) this.ctx.setLineDash([5 / this.scale, 5 / this.scale]);
      } else {
        this.ctx.strokeStyle = theme.connection;
        this.ctx.lineWidth = 1;
        if (this.ctx.setLineDash) this.ctx.setLineDash([]);
      }
      this.ctx.stroke();
    });
    // reset dash after drawing connections
    if (this.ctx.setLineDash) this.ctx.setLineDash([]);

    // Preview connection while dragging
    if (this.draggingConnection) {
      const ac = this.draggingConnection.from.center();
      this.ctx.beginPath();
      this.ctx.moveTo(ac.x, ac.y);
      this.ctx.lineTo(this.draggingConnection.x, this.draggingConnection.y);
      this.ctx.strokeStyle = theme.connection;
      this.ctx.lineWidth = 1;
      this.ctx.stroke();
    }

    // Hover highlights
    drawHoverHighlight(this);

    // Draw blocks
    this.blocks.forEach(b => {
      b.draw(this.ctx);
      if (this.errorBlocks.has(b.id)) {
        this.ctx.strokeStyle = 'red';
        this.ctx.lineWidth = 2;
        this.ctx.strokeRect(b.x, b.y, b.w, b.h);
      }
    });

    // Selection box
    if (this.selectionBox) {
      const { startX, startY, x, y } = this.selectionBox;
      const x1 = Math.min(startX, x);
      const y1 = Math.min(startY, y);
      const w = Math.abs(x - startX);
      const h = Math.abs(y - startY);
      this.ctx.fillStyle = 'rgba(0, 123, 255, 0.2)';
      this.ctx.strokeStyle = 'rgba(0, 123, 255, 0.8)';
      this.ctx.lineWidth = 1 / this.scale;
      this.ctx.fillRect(x1, y1, w, h);
      this.ctx.strokeRect(x1, y1, w, h);
      const sel = new Set();
      for (const b of this.blocks) {
        if (b.x >= x1 && b.x + b.w <= x1 + w && b.y >= y1 && b.y + b.h <= y1 + h) sel.add(b);
      }
      this.selected = sel;
    }

    // Alignment guides
    if (this.alignGuides.length) {
      const width = this.canvas.width / this.scale;
      const height = this.canvas.height / this.scale;
      const startX = -this.offset.x / this.scale;
      const startY = -this.offset.y / this.scale;
      this.ctx.beginPath();
      this.ctx.strokeStyle = theme.alignGuide;
      this.ctx.lineWidth = 1 / this.scale;
      for (const g of this.alignGuides) {
        if (g.type === 'v') {
          this.ctx.moveTo(g.x, startY);
          this.ctx.lineTo(g.x, startY + height);
        } else {
          this.ctx.moveTo(startX, g.y);
          this.ctx.lineTo(startX + width, g.y);
        }
      }
      this.ctx.stroke();
    }

    this.ctx.restore();
    if (this.minimap) this.minimap.render(this);
    requestAnimationFrame(() => this.draw());
  }

  async undo() {
    const action = this.undoStack.pop();
    if (!action) return;
    switch (action.type) {
      case 'move': {
        const block = this.blocks.find(b => b.id === action.id);
        if (block) {
          block.x = action.from.x;
          block.y = action.from.y;
          if (this.moveCallback) await this.moveCallback(block);
        }
        break;
      }
      case 'connect': {
        this.connections = this.connections.filter(
          ([a, b]) => !(a.id === action.from && b.id === action.to)
        );
        this.analyze();
        break;
      }
    }
    this.redoStack.push(action);
  }

  async redo() {
    const action = this.redoStack.pop();
    if (!action) return;
    switch (action.type) {
      case 'move': {
        const block = this.blocks.find(b => b.id === action.id);
        if (block) {
          block.x = action.to.x;
          block.y = action.to.y;
          if (this.moveCallback) await this.moveCallback(block);
        }
        break;
      }
      case 'connect': {
        const from = this.blocks.find(b => b.id === action.from);
        const to = this.blocks.find(b => b.id === action.to);
        if (from && to) {
          this.connections.push([from, to]);
          this.analyze();
        }
        break;
      }
    }
    this.undoStack.push(action);
  }
}

export function exportPNG() {
  const canvas = document.getElementById('visual-canvas');
  if (!(canvas instanceof HTMLCanvasElement)) return;
  const link = document.createElement('a');
  link.href = canvas.toDataURL('image/png');
  link.download = 'canvas.png';
  link.click();
}

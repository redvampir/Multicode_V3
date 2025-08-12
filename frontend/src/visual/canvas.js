import { createBlock } from './blocks.js';
import { getTheme } from './theme.ts';
import { registerHoverHighlight, drawHoverHighlight } from './hover.ts';
import { Minimap } from './minimap.ts';
import settings from '../../settings.json' assert { type: 'json' };

const cfg = settings.visual || {};
const GRID_SIZE = cfg.gridSize || 20;

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
    this.minimap = minimapCanvas ? new Minimap(minimapCanvas) : null;
    this.scale = 1;
    this.offset = { x: 0, y: 0 };
    this.blocks = [];
    this.blocksData = [];
    this.blockDataMap = new Map();
    this.locale = 'en';
    this.connections = [];
    this.debugMode = false;
    this.errorBlocks = new Set();
    this.errorEdges = new Set();
    this.cycleBlocks = new Set();
    this.dragged = null;
    this.dragOffset = { x: 0, y: 0 };
    this.draggingConnection = null;
    this.panning = false;
    this.panStart = { x: 0, y: 0 };
    this.moveCallback = null;
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

    this.resize();
    window.addEventListener('resize', () => this.resize());
    this.registerEvents();
    registerHoverHighlight(this);
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
    if (this.debugMode) this.analyze();
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

  addBlock(block) {
    this.blocks.push(block);
  }

  connect(a, b) {
    this.connections.push([a, b]);
    if (this.debugMode) this.analyze();
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
    this.errorBlocks = missing;
    this.errorEdges = cycles;
    this.cycleBlocks = new Set();
    cycles.forEach(edge => {
      const [from, to] = edge.split('->');
      this.cycleBlocks.add(from);
      this.cycleBlocks.add(to);
    });
  }

  registerEvents() {
    this.canvas.addEventListener('mousedown', e => {
      const pos = this.toWorld(e.offsetX, e.offsetY);
      const block = this.blocks.find(b => b.contains(pos.x, pos.y));

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

    this.canvas.addEventListener('mousemove', e => {
      const pos = this.toWorld(e.offsetX, e.offsetY);
      if (this.draggingConnection) {
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
            if (this.debugMode) {
              if (this.errorBlocks.has(hovered.id)) tooltipText = 'Missing connection';
              else if (this.cycleBlocks.has(hovered.id)) tooltipText = 'Cyclic connection';
            }
          } else if (this.debugMode) {
            for (const edge of this.errorEdges) {
              const [fromId, toId] = edge.split('->');
              const a = this.blocks.find(b => b.id === fromId);
              const b = this.blocks.find(b => b.id === toId);
              if (a && b && this.pointToSegmentDist(pos, a.center(), b.center()) < 5) {
                tooltipText = 'Cyclic connection';
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

    window.addEventListener('mouseup', e => {
      if (this.draggingConnection) {
        const rect = this.canvas.getBoundingClientRect();
        const offX = e.clientX - rect.left;
        const offY = e.clientY - rect.top;
        const pos = this.toWorld(offX, offY);
        const target = this.blocks.find(b => b.contains(pos.x, pos.y));
        if (target && target !== this.draggingConnection.from) {
          this.connections.push([this.draggingConnection.from, target]);
          if (this.debugMode) this.analyze();
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
        if (this.dragged.x !== this.dragStart.x || this.dragged.y !== this.dragStart.y) {
          this.undoStack.push({ id: this.dragged.id, from: { x: this.dragStart.x, y: this.dragStart.y }, to: { x: this.dragged.x, y: this.dragged.y } });
          this.redoStack = [];
        }
        if (this.moveCallback) {
          this.moveCallback(this.dragged);
        }
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
    this.scale = scale;
    const cx = (minX + maxX) / 2;
    const cy = (minY + maxY) / 2;
    this.offset.x = this.canvas.width / 2 - cx * scale;
    this.offset.y = this.canvas.height / 2 - cy * scale;
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
    const lines = [];
    if (!this.dragged) {
      this.alignGuides = lines;
      return;
    }
    const threshold = 5;
    const d = this.dragged;
    const dxs = [d.x, d.x + d.w / 2, d.x + d.w];
    const dys = [d.y, d.y + d.h / 2, d.y + d.h];
    for (const b of this.blocks) {
      if (b === d) continue;
      const bx = [b.x, b.x + b.w / 2, b.x + b.w];
      const by = [b.y, b.y + b.h / 2, b.y + b.h];
      for (const x1 of dxs) {
        for (const x2 of bx) {
          if (Math.abs(x1 - x2) < threshold) {
            lines.push({ type: 'v', x: x2 });
          }
        }
      }
      for (const y1 of dys) {
        for (const y2 of by) {
          if (Math.abs(y1 - y2) < threshold) {
            lines.push({ type: 'h', y: y2 });
          }
        }
      }
    }
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
      if (this.debugMode && this.errorEdges.has(key)) {
        this.ctx.strokeStyle = 'red';
        this.ctx.lineWidth = 2;
      } else {
        this.ctx.strokeStyle = theme.connection;
        this.ctx.lineWidth = 1;
      }
      this.ctx.stroke();
    });

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
      if (this.debugMode && this.errorBlocks.has(b.id)) {
        this.ctx.strokeStyle = 'red';
        this.ctx.lineWidth = 2;
        this.ctx.strokeRect(b.x, b.y, b.w, b.h);
      }
    });

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
    if (action) {
      const block = this.blocks.find(b => b.id === action.id);
      if (block) {
        block.x = action.from.x;
        block.y = action.from.y;
        if (this.moveCallback) await this.moveCallback(block);
      }
      this.redoStack.push(action);
    }
  }

  async redo() {
    const action = this.redoStack.pop();
    if (action) {
      const block = this.blocks.find(b => b.id === action.id);
      if (block) {
        block.x = action.to.x;
        block.y = action.to.y;
        if (this.moveCallback) await this.moveCallback(block);
      }
      this.undoStack.push(action);
    }
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

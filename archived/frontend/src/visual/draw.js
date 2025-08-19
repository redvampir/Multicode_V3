import { getTheme } from './theme.ts';
import { drawHoverHighlight } from './hover.ts';
import { GRID_SIZE } from './settings.ts';
import { GroupBlock } from './blocks.js';

export function renderCanvas(vc) {
  const theme = getTheme();
  vc.ctx.save();
  vc.ctx.setTransform(vc.scale, 0, 0, vc.scale, vc.offset.x, vc.offset.y);
  vc.ctx.clearRect(-vc.offset.x / vc.scale, -vc.offset.y / vc.scale,
    vc.canvas.width / vc.scale, vc.canvas.height / vc.scale);

  if (vc.gridEnabled) {
    const size = GRID_SIZE;
    const width = vc.canvas.width / vc.scale;
    const height = vc.canvas.height / vc.scale;
    const startX = Math.floor((-vc.offset.x / vc.scale) / size) * size;
    const startY = Math.floor((-vc.offset.y / vc.scale) / size) * size;
    const endX = startX + width + size;
    const endY = startY + height + size;
    vc.ctx.beginPath();
    vc.ctx.strokeStyle = '#eee';
    vc.ctx.lineWidth = 1 / vc.scale;
    for (let x = startX; x <= endX; x += size) {
      vc.ctx.moveTo(x, startY);
      vc.ctx.lineTo(x, endY);
    }
    for (let y = startY; y <= endY; y += size) {
      vc.ctx.moveTo(startX, y);
      vc.ctx.lineTo(endX, y);
    }
    vc.ctx.stroke();
  }

  // Draw groups
  for (const group of vc.groups.values()) {
    const members = vc.blocks.filter(b => group.blocks.has(b.id));
    if (!members.length) continue;
    const padding = 10;
    const minX = Math.min(...members.map(b => b.x)) - padding;
    const minY = Math.min(...members.map(b => b.y)) - padding;
    const maxX = Math.max(...members.map(b => b.x + b.w)) + padding;
    const maxY = Math.max(...members.map(b => b.y + b.h)) + padding;
    const gb = new GroupBlock('', minX, minY, maxX - minX, maxY - minY, group.label, group.color);
    gb.draw(vc.ctx);
  }

  // Draw connections
  vc.connections.forEach(([a, b]) => {
    const ac = a.center();
    const bc = b.center();
    const key = a.id + '->' + b.id;
    vc.ctx.beginPath();
    vc.ctx.moveTo(ac.x, ac.y);
    vc.ctx.lineTo(bc.x, bc.y);
    if (vc.errorEdges.has(key)) {
      vc.ctx.strokeStyle = 'red';
      vc.ctx.lineWidth = 2;
      if (vc.ctx.setLineDash) vc.ctx.setLineDash([]);
    } else if (vc.missingEdge === key) {
      vc.ctx.strokeStyle = 'orange';
      vc.ctx.lineWidth = 2;
      if (vc.ctx.setLineDash) vc.ctx.setLineDash([5 / vc.scale, 5 / vc.scale]);
    } else {
      vc.ctx.strokeStyle = theme.connection;
      vc.ctx.lineWidth = 1;
      if (vc.ctx.setLineDash) vc.ctx.setLineDash([]);
    }
    vc.ctx.stroke();
  });
  // reset dash after drawing connections
  if (vc.ctx.setLineDash) vc.ctx.setLineDash([]);

  // Preview connection while dragging
  if (vc.draggingConnection) {
    const ac = vc.draggingConnection.from.center();
    vc.ctx.beginPath();
    vc.ctx.moveTo(ac.x, ac.y);
    vc.ctx.lineTo(vc.draggingConnection.x, vc.draggingConnection.y);
    vc.ctx.strokeStyle = theme.connection;
    vc.ctx.lineWidth = 1;
    vc.ctx.stroke();
  }

  // Hover highlights
  drawHoverHighlight(vc);

  // Draw blocks
  vc.blocks.forEach(b => {
    b.draw(vc.ctx);
    if (vc.testResults.has(b.id)) {
      const ok = vc.testResults.get(b.id);
      vc.ctx.strokeStyle = ok ? 'green' : 'red';
      vc.ctx.lineWidth = 2;
      vc.ctx.strokeRect(b.x, b.y, b.w, b.h);
      if (!ok) {
        vc.ctx.fillStyle = 'red';
        vc.ctx.font = `${12 / vc.scale}px sans-serif`;
        vc.ctx.fillText('!', b.x + 4 / vc.scale, b.y + 14 / vc.scale);
      }
    } else if (vc.errorBlocks.has(b.id)) {
      vc.ctx.strokeStyle = 'red';
      vc.ctx.lineWidth = 2;
      vc.ctx.strokeRect(b.x, b.y, b.w, b.h);
      vc.ctx.fillStyle = 'orange';
      vc.ctx.font = `${12 / vc.scale}px sans-serif`;
      vc.ctx.fillText('⚠', b.x + 4 / vc.scale, b.y + 14 / vc.scale);
    }
  });

  // Selection box
  if (vc.selectionBox) {
    const { startX, startY, x, y } = vc.selectionBox;
    const x1 = Math.min(startX, x);
    const y1 = Math.min(startY, y);
    const w = Math.abs(x - startX);
    const h = Math.abs(y - startY);
    vc.ctx.fillStyle = 'rgba(0, 123, 255, 0.2)';
    vc.ctx.strokeStyle = 'rgba(0, 123, 255, 0.8)';
    vc.ctx.lineWidth = 1 / vc.scale;
    vc.ctx.fillRect(x1, y1, w, h);
    vc.ctx.strokeRect(x1, y1, w, h);
    const sel = new Set();
    for (const b of vc.blocks) {
      if (b.x >= x1 && b.x + b.w <= x1 + w && b.y >= y1 && b.y + b.h <= y1 + h) sel.add(b);
    }
    vc.selected = sel;
  }

  // Alignment guides
  if (vc.alignGuides.length) {
    const width = vc.canvas.width / vc.scale;
    const height = vc.canvas.height / vc.scale;
    const startX = -vc.offset.x / vc.scale;
    const startY = -vc.offset.y / vc.scale;
    vc.ctx.beginPath();
    vc.ctx.strokeStyle = theme.alignGuide;
    vc.ctx.lineWidth = 1 / vc.scale;
    for (const g of vc.alignGuides) {
      if (g.type === 'v') {
        vc.ctx.moveTo(g.x, startY);
        vc.ctx.lineTo(g.x, startY + height);
      } else {
        vc.ctx.moveTo(startX, g.y);
        vc.ctx.lineTo(startX + width, g.y);
      }
    }
    vc.ctx.stroke();
  }

  vc.ctx.restore();
  if (vc.minimap) vc.minimap.render(vc);
}

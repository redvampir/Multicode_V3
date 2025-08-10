import { getTheme } from './theme.ts';

// Minimal interface of VisualCanvas used in this module.
export interface HoverCanvas {
  canvas: HTMLCanvasElement;
  connections: Array<[any, any]>;
  blocks: any[];
  hovered: any | null;
  ctx: CanvasRenderingContext2D;
  toWorld(x: number, y: number): { x: number; y: number };
}

/**
 * Register listeners to track hovered block on the canvas.
 */
export function registerHoverHighlight(vc: HoverCanvas) {
  vc.hovered = null;
  vc.canvas.addEventListener('mousemove', e => {
    const pos = vc.toWorld(e.offsetX, e.offsetY);
    vc.hovered = vc.blocks.find(b => b.contains(pos.x, pos.y)) || null;
  });
  vc.canvas.addEventListener('mouseleave', () => {
    vc.hovered = null;
  });
}

/**
 * Draw highlighted connections for the hovered block.
 */
export function drawHoverHighlight(vc: HoverCanvas) {
  if (!vc.hovered) return;
  const theme = getTheme();
  vc.connections.forEach(([a, b]) => {
    if (a === vc.hovered || b === vc.hovered) {
      const ac = a.center();
      const bc = b.center();
      vc.ctx.save();
      vc.ctx.beginPath();
      vc.ctx.moveTo(ac.x, ac.y);
      vc.ctx.lineTo(bc.x, bc.y);
      vc.ctx.strokeStyle = theme.highlight;
      vc.ctx.lineWidth = 3;
      vc.ctx.stroke();
      vc.ctx.restore();
    }
  });
}


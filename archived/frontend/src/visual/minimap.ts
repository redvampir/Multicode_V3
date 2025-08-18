import type { VisualCanvas } from './canvas.js';
import { getTheme } from './theme.ts';

export class Minimap {
  canvas: HTMLCanvasElement;
  ctx: CanvasRenderingContext2D;
  vc: VisualCanvas;
  private dragging = false;

  constructor(canvas: HTMLCanvasElement, vc: VisualCanvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d')!;
    this.vc = vc;

    const updateView = (e: MouseEvent) => {
      const blocks = this.vc.blocks;
      if (!blocks || blocks.length === 0) return;

      const width = this.canvas.width;
      const height = this.canvas.height;
      let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
      for (const b of blocks) {
        minX = Math.min(minX, b.x);
        minY = Math.min(minY, b.y);
        maxX = Math.max(maxX, b.x + b.w);
        maxY = Math.max(maxY, b.y + b.h);
      }
      const worldW = maxX - minX || 1;
      const worldH = maxY - minY || 1;
      const scale = Math.min(width / worldW, height / worldH);
      const originX = -minX * scale + (width - worldW * scale) / 2;
      const originY = -minY * scale + (height - worldH * scale) / 2;

      const worldX = (e.offsetX - originX) / scale;
      const worldY = (e.offsetY - originY) / scale;
      const viewW = this.vc.canvas.width / this.vc.scale;
      const viewH = this.vc.canvas.height / this.vc.scale;
      this.vc.offset.x = -(worldX - viewW / 2) * this.vc.scale;
      this.vc.offset.y = -(worldY - viewH / 2) * this.vc.scale;
      if (typeof this.vc.draw === 'function') this.vc.draw();
    };

    canvas.addEventListener('mousedown', e => {
      this.dragging = true;
      updateView(e);
    });
    canvas.addEventListener('mousemove', e => {
      if (this.dragging) updateView(e);
    });
    const stop = () => { this.dragging = false; };
    window.addEventListener('mouseup', stop);
    canvas.addEventListener('mouseleave', stop);
  }

  render(vc: VisualCanvas) {
    const ctx = this.ctx;
    const width = this.canvas.width;
    const height = this.canvas.height;
    ctx.clearRect(0, 0, width, height);

    const blocks = vc.blocks;
    if (!blocks || blocks.length === 0) return;

    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    for (const b of blocks) {
      minX = Math.min(minX, b.x);
      minY = Math.min(minY, b.y);
      maxX = Math.max(maxX, b.x + b.w);
      maxY = Math.max(maxY, b.y + b.h);
    }
    const worldW = maxX - minX || 1;
    const worldH = maxY - minY || 1;
    const scale = Math.min(width / worldW, height / worldH);
    const originX = -minX * scale + (width - worldW * scale) / 2;
    const originY = -minY * scale + (height - worldH * scale) / 2;

    const theme = getTheme();
    ctx.fillStyle = theme.blockFill;
    for (const b of blocks) {
      ctx.fillRect(b.x * scale + originX, b.y * scale + originY, b.w * scale, b.h * scale);
    }

    const viewX = -vc.offset.x / vc.scale;
    const viewY = -vc.offset.y / vc.scale;
    const viewW = vc.canvas.width / vc.scale;
    const viewH = vc.canvas.height / vc.scale;

    ctx.strokeStyle = theme.connection;
    ctx.lineWidth = 1;
    ctx.strokeRect(viewX * scale + originX, viewY * scale + originY, viewW * scale, viewH * scale);
  }
}

import type { VisualCanvas } from './canvas.js';

export const EXPORT_PADDING = 20;

let canvasRef: VisualCanvas | null = null;

export function setCanvas(vc: VisualCanvas) {
  canvasRef = vc;
}

export function renderToCanvas(vc: VisualCanvas): HTMLCanvasElement | OffscreenCanvas {
  if (!vc.blocks.length) {
    const empty = typeof OffscreenCanvas === 'function'
      ? new OffscreenCanvas(1, 1)
      : document.createElement('canvas');
    empty.width = 1;
    empty.height = 1;
    return empty as any;
  }
  const minX = Math.min(...vc.blocks.map(b => b.x));
  const minY = Math.min(...vc.blocks.map(b => b.y));
  const maxX = Math.max(...vc.blocks.map(b => b.x + b.w));
  const maxY = Math.max(...vc.blocks.map(b => b.y + b.h));
  const width = maxX - minX + EXPORT_PADDING * 2;
  const height = maxY - minY + EXPORT_PADDING * 2;
  const off: any = typeof OffscreenCanvas === 'function'
    ? new OffscreenCanvas(width, height)
    : document.createElement('canvas');
  off.width = width;
  off.height = height;
  const ctx = off.getContext('2d');
  const prevCanvas = vc.canvas;
  const prevCtx = vc.ctx;
  const prevScale = vc.scale;
  const prevOffset = { ...vc.offset };
  vc.canvas = off as any;
  vc.ctx = ctx as any;
  vc.scale = 1;
  vc.offset = { x: -minX + EXPORT_PADDING, y: -minY + EXPORT_PADDING };
  vc.draw();
  vc.canvas = prevCanvas;
  vc.ctx = prevCtx;
  vc.scale = prevScale;
  vc.offset = prevOffset;
  return off;
}

export async function exportPNG() {
  if (!canvasRef) return;
  const off = renderToCanvas(canvasRef);
  let blob: Blob;
  if (off instanceof OffscreenCanvas) {
    blob = await off.convertToBlob({ type: 'image/png' });
  } else {
    blob = await new Promise<Blob>(resolve => (off as HTMLCanvasElement).toBlob(b => resolve(b!), 'image/png'));
  }
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = 'canvas.png';
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}


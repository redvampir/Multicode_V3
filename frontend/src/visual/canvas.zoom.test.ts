// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../editor/visual-meta.js', () => ({
  updateMetaComment: vi.fn(),
  previewDiff: vi.fn().mockResolvedValue(true),
  renameMetaId: vi.fn().mockResolvedValue(true)
}));
import { VisualCanvas } from './canvas.js';

function createCtx() {
  return {
    save: vi.fn(),
    restore: vi.fn(),
    setTransform: vi.fn(),
    clearRect: vi.fn(),
    beginPath: vi.fn(),
    stroke: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    strokeRect: vi.fn(),
    fillRect: vi.fn(),
    fillText: vi.fn(),
    drawImage: vi.fn(),
    arc: vi.fn(),
    clip: vi.fn(),
    translate: vi.fn(),
    scale: vi.fn(),
    setLineDash: vi.fn()
  } as any;
}

function createCanvas(ctx: any) {
  const canvasEl = document.createElement('canvas');
  Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
  Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
  canvasEl.getContext = () => ctx;
  return canvasEl as HTMLCanvasElement;
}

describe('canvas zoom interactions', () => {
  it('updates scale on wheel', () => {
    const ctx = createCtx();
    const canvasEl = createCanvas(ctx);
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    const wheel = new WheelEvent('wheel', { deltaY: -100 });
    Object.defineProperty(wheel, 'offsetX', { get: () => 100 });
    Object.defineProperty(wheel, 'offsetY', { get: () => 100 });
    const prev = vc.scale;
    canvasEl.dispatchEvent(wheel);
    expect(vc.scale).toBeGreaterThan(prev);
  });

  it('toggles magnifier with Alt key', () => {
    const ctx = createCtx();
    const canvasEl = createCanvas(ctx);
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Alt' }));
    expect(vc.magnifier.active).toBe(true);
    window.dispatchEvent(new KeyboardEvent('keyup', { key: 'Alt' }));
    expect(vc.magnifier.active).toBe(false);
  });

  it('scales micro blocks under magnifier', () => {
    const ctx = createCtx();
    const canvasEl = createCanvas(ctx);
    ctx.scale = vi.fn();
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    vc.blocks = [{ id: 'm', x: 0, y: 0, w: 56, h: 28, draw: vi.fn(), contains(){return false;} }];
    vc.magnifier.active = true;
    vc.magnifier.x = 50; vc.magnifier.y = 50;
    vc.gridEnabled = false;
    vc.draw();
    expect(ctx.scale).toHaveBeenCalled();
  });
});

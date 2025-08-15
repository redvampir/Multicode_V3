// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
import { Minimap } from './minimap.ts';

function createCtx() {
  return {
    clearRect: vi.fn(),
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillStyle: '',
    strokeStyle: '',
    lineWidth: 0
  } as any;
}

function createVc(blocks) {
  return {
    canvas: { width: 200, height: 200 },
    scale: 1,
    offset: { x: 0, y: 0 },
    blocks,
    draw: vi.fn()
  } as any;
}

describe('Minimap', () => {
  it('renders blocks and view frame', () => {
    const mini = document.createElement('canvas');
    mini.width = 100; mini.height = 100;
    const ctx = createCtx();
    mini.getContext = () => ctx;
    const vc = createVc([
      { x: 0, y: 0, w: 50, h: 50 },
      { x: 150, y: 150, w: 50, h: 50 }
    ]);
    const m = new Minimap(mini, vc);
    m.render(vc);
    expect(ctx.fillRect).toHaveBeenCalledWith(0, 0, 25, 25);
    expect(ctx.fillRect).toHaveBeenCalledWith(75, 75, 25, 25);
    expect(ctx.strokeRect).toHaveBeenCalledWith(0, 0, 100, 100);
  });

  it('recenters view on click', () => {
    const mini = document.createElement('canvas');
    mini.width = 100; mini.height = 100;
    const ctx = createCtx();
    mini.getContext = () => ctx;
    const vc = createVc([
      { x: 0, y: 0, w: 50, h: 50 },
      { x: 150, y: 150, w: 50, h: 50 }
    ]);
    new Minimap(mini, vc);
    const e = new MouseEvent('mousedown', { clientX: 0, clientY: 0 });
    Object.defineProperty(e, 'offsetX', { get: () => 100 });
    Object.defineProperty(e, 'offsetY', { get: () => 100 });
    mini.dispatchEvent(e);
    expect(vc.offset.x).toBe(-100);
    expect(vc.offset.y).toBe(-100);
  });

  it('updates view while dragging', () => {
    const mini = document.createElement('canvas');
    mini.width = 100; mini.height = 100;
    const ctx = createCtx();
    mini.getContext = () => ctx;
    const vc = createVc([
      { x: 0, y: 0, w: 50, h: 50 },
      { x: 150, y: 150, w: 50, h: 50 }
    ]);
    new Minimap(mini, vc);
    const down = new MouseEvent('mousedown', { clientX: 0, clientY: 0 });
    Object.defineProperty(down, 'offsetX', { get: () => 0 });
    Object.defineProperty(down, 'offsetY', { get: () => 0 });
    mini.dispatchEvent(down);
    const move = new MouseEvent('mousemove', { clientX: 0, clientY: 0 });
    Object.defineProperty(move, 'offsetX', { get: () => 100 });
    Object.defineProperty(move, 'offsetY', { get: () => 100 });
    mini.dispatchEvent(move);
    expect(vc.offset.x).toBe(-100);
    expect(vc.offset.y).toBe(-100);
  });
});

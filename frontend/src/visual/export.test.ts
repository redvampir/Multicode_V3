// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { renderToCanvas, EXPORT_PADDING } from './export.ts';

describe('export', () => {
  it('calculates canvas size to include edge blocks', () => {
    class StubOffscreen {
      width: number;
      height: number;
      constructor(w: number, h: number) { this.width = w; this.height = h; }
      getContext() { return {}; }
    }
    const old = (globalThis as any).OffscreenCanvas;
    (globalThis as any).OffscreenCanvas = StubOffscreen as any;
    let seenOffset: any;
    const vc: any = {
      blocks: [
        { x: 0, y: 0, w: 10, h: 10 },
        { x: 100, y: 100, w: 10, h: 10 }
      ],
      canvas: { width: 0, height: 0 },
      ctx: {},
      scale: 1,
      offset: { x: 0, y: 0 },
      draw() { seenOffset = { ...this.offset }; }
    };
    const off = renderToCanvas(vc) as any;
    (globalThis as any).OffscreenCanvas = old;
    expect(off.width).toBe(150);
    expect(off.height).toBe(150);
    const dx = seenOffset.x;
    const dy = seenOffset.y;
    for (const b of vc.blocks) {
      const left = b.x + dx;
      const right = left + b.w;
      const top = b.y + dy;
      const bottom = top + b.h;
      expect(left).toBeGreaterThanOrEqual(0);
      expect(top).toBeGreaterThanOrEqual(0);
      expect(right).toBeLessThanOrEqual(off.width);
      expect(bottom).toBeLessThanOrEqual(off.height);
    }
    expect(dx).toBe(EXPORT_PADDING);
    expect(dy).toBe(EXPORT_PADDING);
  });
});


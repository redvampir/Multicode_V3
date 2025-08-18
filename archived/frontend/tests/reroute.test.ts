/** @vitest-environment jsdom */
import { describe, it, expect, beforeEach, vi } from 'vitest';

vi.mock('@codemirror/state', () => ({
  StateField: { define: vi.fn() },
  RangeSetBuilder: class {},
  StateEffect: { define: vi.fn(() => ({ of: vi.fn() })) }
}));

vi.mock('@codemirror/view', () => ({
  Decoration: { mark: () => ({}) },
  EditorView: { decorations: { from: vi.fn() }, updateListener: { of: vi.fn() }, baseTheme: vi.fn() }
}));

vi.mock('@codemirror/language', () => ({
  hoverTooltip: () => ({})
}));

import { VisualCanvas } from '../src/visual/canvas.js';
import { Block } from '../src/visual/blocks.js';

function createCanvas() {
  const canvas = document.createElement('canvas');
  const ctx = {
    canvas,
    save() {},
    restore() {},
    setTransform() {},
    clearRect() {},
    beginPath() {},
    moveTo() {},
    lineTo() {},
    stroke() {},
    fillRect() {},
    strokeRect() {},
    fillText() {},
    arc() {},
    fill() {},
    setLineDash() {},
    translate() {},
    scale() {},
  } as any;
  canvas.getContext = () => ctx;
  return canvas;
}

describe('reroute insertion', () => {
  beforeEach(() => {
    (global as any).requestAnimationFrame = () => 0;
  });

  it('inserts reroute block on double click', () => {
    const canvasEl = createCanvas();
    const vc = new VisualCanvas(canvasEl);
    const a = new Block('a', 0, 0, 120, 50, 'A');
    const b = new Block('b', 200, 0, 120, 50, 'B');
    vc.addBlock(a);
    vc.addBlock(b);
    vc.connections.push([a, b]);

    const evt = new MouseEvent('dblclick', { bubbles: true });
    Object.defineProperty(evt, 'offsetX', { value: 160 });
    Object.defineProperty(evt, 'offsetY', { value: 25 });
    canvasEl.dispatchEvent(evt);

    expect(vc.blocks.length).toBe(3);
    const reroute = vc.blocks.find(bl => bl.id !== 'a' && bl.id !== 'b');
    expect(reroute).toBeTruthy();
    expect(vc.connections).toHaveLength(2);
    expect(vc.connections[0][0]).toBe(a);
    expect(vc.connections[1][1]).toBe(b);
  });
});

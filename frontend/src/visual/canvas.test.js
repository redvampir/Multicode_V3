// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { analyzeConnections, VisualCanvas } from './canvas.js';

describe('analyzeConnections', () => {
  it('detects missing blocks and cycles', () => {
    const blocks = ['a', 'b', 'c'];
    const edges = [['a', 'b'], ['b', 'a']];
    const { missing, cycles } = analyzeConnections(blocks, edges);
    expect(Array.from(missing)).toEqual(['c']);
    expect(cycles.has('a->b')).toBe(true);
    expect(cycles.has('b->a')).toBe(true);
  });
});

describe('zoomToFit', () => {
  it('adjusts scale and offset to contain all blocks', () => {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    vc.blocks = [
      { x: 0, y: 0, w: 50, h: 50 },
      { x: 150, y: 150, w: 50, h: 50 }
    ];
    vc.zoomToFit();
    const minX = 0; const minY = 0; const maxX = 200; const maxY = 200;
    const topLeft = { x: minX * vc.scale + vc.offset.x, y: minY * vc.scale + vc.offset.y };
    const bottomRight = { x: maxX * vc.scale + vc.offset.x, y: maxY * vc.scale + vc.offset.y };
    expect(topLeft.x).toBeGreaterThanOrEqual(0);
    expect(topLeft.y).toBeGreaterThanOrEqual(0);
    expect(bottomRight.x).toBeLessThanOrEqual(canvasEl.width);
    expect(bottomRight.y).toBeLessThanOrEqual(canvasEl.height);
  });
});

describe('selection box', () => {
  it('selects blocks inside the rectangle', () => {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    const vc = new VisualCanvas(canvasEl);
    vc.blocks = [
      { id: 'a', x: 10, y: 10, w: 20, h: 20, draw(){} },
      { id: 'b', x: 100, y: 100, w: 20, h: 20, draw(){} }
    ];
    vc.selectionBox = { startX: 0, startY: 0, x: 50, y: 50 };
    vc.draw();
    expect(vc.selected.size).toBe(1);
    expect(Array.from(vc.selected)[0].id).toBe('a');
  });
});

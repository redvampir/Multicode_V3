// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../editor/visual-meta.js', () => ({
  updateMetaComment: vi.fn(),
  previewDiff: vi.fn().mockResolvedValue(true),
  renameMetaId: vi.fn().mockResolvedValue(true)
}));
import { analyzeConnections, VisualCanvas } from './canvas.js';
import { renameMetaId } from '../editor/visual-meta.js';

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

describe('undo and redo', () => {
  function createCanvas() {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    return canvasEl;
  }

  it('undoes and redoes block movement', async () => {
    const vc = new VisualCanvas(createCanvas());
    vc.blocks = [{ id: 'a', x: 0, y: 0 }];
    vc.undoStack.push({ type: 'move', id: 'a', from: { x: 0, y: 0 }, to: { x: 10, y: 20 } });
    vc.blocks[0].x = 10;
    vc.blocks[0].y = 20;
    await vc.undo();
    expect(vc.blocks[0].x).toBe(0);
    expect(vc.blocks[0].y).toBe(0);
    await vc.redo();
    expect(vc.blocks[0].x).toBe(10);
    expect(vc.blocks[0].y).toBe(20);
  });

  it('undoes and redoes connections', async () => {
    const vc = new VisualCanvas(createCanvas());
    const a = { id: 'a' };
    const b = { id: 'b' };
    vc.blocks = [a, b];
    vc.connections = [[a, b]];
    vc.undoStack.push({ type: 'connect', from: 'a', to: 'b' });
    await vc.undo();
    expect(vc.connections.length).toBe(0);
    await vc.redo();
    expect(vc.connections.length).toBe(1);
    expect(vc.connections[0][0]).toBe(a);
    expect(vc.connections[0][1]).toBe(b);
  });
});

describe('renameBlock', () => {
  function createCanvas() {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    return canvasEl;
  }

  it('renames block id and data', async () => {
    const vc = new VisualCanvas(createCanvas());
    vc.blocks = [{ id: 'old' }];
    const data = { visual_id: 'old', kind: 'Fn', x: 0, y: 0 };
    vc.blocksData = [data];
    vc.blockDataMap.set('old', data);
    vc.metaView = {};
    globalThis.prompt = vi.fn().mockReturnValue('new');
    await vc.renameBlock('old');
    expect(vc.blocks[0].id).toBe('new');
    expect(vc.blockDataMap.has('new')).toBe(true);
    expect(vc.blocksData[0].visual_id).toBe('new');
    expect(renameMetaId).toHaveBeenCalledWith(vc.metaView, 'old', 'new');
  });
});

describe('serialize and load', () => {
  function createCanvas() {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({ save(){}, setTransform(){}, clearRect(){}, beginPath(){}, stroke(){}, moveTo(){}, lineTo(){}, fillRect(){}, strokeRect(){}, fillText(){}, restore(){} });
    globalThis.requestAnimationFrame = () => 0;
    return canvasEl;
  }

  it('restores blocks, connections and view', () => {
    const vc1 = new VisualCanvas(createCanvas());
    const a = { id: 'a' };
    const b = { id: 'b' };
    vc1.blocks = [a, b];
    vc1.blocksData = [
      { visual_id: 'a', kind: 'Function', x: 0, y: 0 },
      { visual_id: 'b', kind: 'Function', x: 10, y: 10 }
    ];
    vc1.connections = [[a, b]];
    vc1.offset = { x: 5, y: 6 };
    vc1.scale = 2;
    vc1.groups.set(1, { blocks: new Set(['a', 'b']), color: '#123456', label: 'g1' });

    const data = vc1.serialize();

    const vc2 = new VisualCanvas(createCanvas());
    vc2.load(data);

    expect(vc2.blocksData).toEqual(vc1.blocksData);
    expect(vc2.connections.length).toBe(1);
    expect(vc2.connections[0][0].id).toBe('a');
    expect(vc2.connections[0][1].id).toBe('b');
    expect(vc2.offset).toEqual({ x: 5, y: 6 });
    expect(vc2.scale).toBe(2);
    expect(vc2.groups.size).toBe(1);
    const g = vc2.groups.get(1);
    expect(g?.color).toBe('#123456');
    expect(Array.from(g?.blocks || [])).toEqual(['a', 'b']);
  });
});

import test from 'node:test';
import assert from 'node:assert';
import {
  initCanvas,
  attachEventHandlers,
  addBlock,
  removeBlock,
  diffBlocks,
  drawBlock,
  createCanvasApp
} from './canvas.js';

class MockCanvas {
  constructor() {
    this.width = 100;
    this.height = 100;
    this.listeners = {};
    this.ctx = {
      fillRect: () => { this.fillCalled = true; },
      clearRect: () => { this.clearCalled = true; }
    };
  }
  getContext() { return this.ctx; }
  addEventListener(type, handler) { this.listeners[type] = handler; }
  removeEventListener(type) { delete this.listeners[type]; }
  dispatchEvent(ev) { const h = this.listeners[ev.type]; if (h) h(ev); }
}

test('initCanvas validates input', () => {
  assert.throws(() => initCanvas({}), /getContext/);
  const canvas = new MockCanvas();
  const { state } = initCanvas(canvas);
  assert.deepStrictEqual(state.blocks, []);
});

test('event handlers record events', () => {
  const canvas = new MockCanvas();
  const { state } = initCanvas(canvas);
  attachEventHandlers(canvas, state);
  canvas.dispatchEvent({ type: 'mousedown' });
  canvas.dispatchEvent({ type: 'keydown' });
  assert.strictEqual(state.events.length, 2);
});

test('block operations validate input', () => {
  const state = { blocks: [] };
  assert.throws(() => addBlock(state, {}), /block.id/);
  addBlock(state, { id: 'a', x:0, y:0, w:1, h:1 });
  assert.strictEqual(state.blocks.length, 1);
  const removed = removeBlock(state, 'a');
  assert.strictEqual(removed.id, 'a');
  assert.strictEqual(state.blocks.length, 0);
  assert.throws(() => removeBlock(state, 'missing'), /block not found/);
});

test('diffBlocks reports changes', () => {
  const a = { id: 'a', x: 0, y: 0, w: 1, h: 1 };
  const b = { id: 'a', x: 5, y: 0, w: 1, h: 1 };
  const diff = diffBlocks(a, b);
  assert.deepStrictEqual(diff, { x: { from: 0, to: 5 } });
});

test('drawBlock uses canvas context', () => {
  const canvas = new MockCanvas();
  const { ctx } = initCanvas(canvas);
  drawBlock(ctx, { x: 1, y: 2, w: 3, h: 4 });
  assert.ok(canvas.fillCalled);
});

test('createCanvasApp wires modules together', () => {
  const canvas = new MockCanvas();
  const app = createCanvasApp(canvas);
  app.addBlock({ id: 'x', x:0, y:0, w:1, h:1 });
  assert.strictEqual(app.state.blocks.length, 1);
  canvas.dispatchEvent({ type: 'mousedown' });
  assert.strictEqual(app.state.events.length, 1);
  app.clear();
  assert.ok(canvas.clearCalled);
});

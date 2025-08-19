import { initCanvas } from './init.js';
import { attachEventHandlers } from './events.js';
import { addBlock, removeBlock, diffBlocks } from './blocks.js';
import { drawBlock, clearCanvas } from './helpers.js';

export {
  initCanvas,
  attachEventHandlers,
  addBlock,
  removeBlock,
  diffBlocks,
  drawBlock,
  clearCanvas
};

export function createCanvasApp(canvas) {
  const { ctx, state } = initCanvas(canvas);
  attachEventHandlers(canvas, state);
  return {
    canvas,
    ctx,
    state,
    addBlock: block => addBlock(state, block),
    removeBlock: id => removeBlock(state, id),
    diffBlocks,
    drawBlock: block => drawBlock(ctx, block),
    clear: () => clearCanvas(canvas, ctx)
  };
}

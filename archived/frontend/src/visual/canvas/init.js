export function initCanvas(canvas) {
  if (!canvas || typeof canvas.getContext !== 'function') {
    throw new TypeError('A canvas with getContext() is required');
  }
  const ctx = canvas.getContext('2d');
  const state = { blocks: [], events: [] };
  return { canvas, ctx, state };
}

export function drawBlock(ctx, block) {
  if (!ctx || typeof ctx.fillRect !== 'function') {
    throw new TypeError('ctx with fillRect() required');
  }
  if (!block) throw new TypeError('block required');
  ctx.fillRect(block.x, block.y, block.w, block.h);
}

export function clearCanvas(canvas, ctx) {
  ctx.clearRect(0, 0, canvas.width, canvas.height);
}

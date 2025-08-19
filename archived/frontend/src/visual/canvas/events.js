export function attachEventHandlers(canvas, state) {
  if (!canvas || typeof canvas.addEventListener !== 'function') {
    throw new TypeError('A valid canvas with event support is required');
  }
  const onMouse = e => state.events.push({ type: e.type });
  const onKey = e => state.events.push({ type: e.type });
  canvas.addEventListener('mousedown', onMouse);
  canvas.addEventListener('keydown', onKey);
  return {
    detach() {
      canvas.removeEventListener('mousedown', onMouse);
      canvas.removeEventListener('keydown', onKey);
    }
  };
}

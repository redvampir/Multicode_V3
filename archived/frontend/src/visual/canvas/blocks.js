export function addBlock(state, block) {
  if (!block || typeof block.id !== 'string') {
    throw new TypeError('block.id string required');
  }
  state.blocks.push(block);
  return block;
}

export function removeBlock(state, id) {
  const idx = state.blocks.findIndex(b => b.id === id);
  if (idx === -1) throw new Error('block not found');
  return state.blocks.splice(idx, 1)[0];
}

export function diffBlocks(a, b) {
  if (!a || !b) throw new TypeError('two blocks required');
  const diff = {};
  for (const key of ['id', 'x', 'y', 'w', 'h']) {
    if (a[key] !== b[key]) diff[key] = { from: a[key], to: b[key] };
  }
  return diff;
}

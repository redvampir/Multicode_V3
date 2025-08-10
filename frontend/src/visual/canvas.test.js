import { describe, it, expect } from 'vitest';
import { analyzeConnections } from './canvas.js';

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

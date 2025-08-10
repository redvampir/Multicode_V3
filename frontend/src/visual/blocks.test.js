import { describe, it, expect } from 'vitest';
import { Block, registerBlock, createBlock } from './blocks.js';

describe('block utilities', () => {
  it('checks point containment and center', () => {
    const b = new Block('1', 0, 0, 10, 10, 'test');
    expect(b.contains(5,5)).toBe(true);
    expect(b.center()).toEqual({ x: 5, y: 5 });
  });

  it('registers and creates custom block', () => {
    class Custom extends Block {
      constructor(id, x, y) { super(id, x, y, 10, 10, 'c'); }
    }
    registerBlock('custom', Custom);
    const b = createBlock('custom', '2', 0, 0, 'c');
    expect(b).toBeInstanceOf(Custom);
  });
});

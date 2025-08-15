import { describe, it, expect } from 'vitest';
import {
  Block,
  registerBlock,
  unregisterBlock,
  createBlock,
  NumberLiteralBlock,
  StringLiteralBlock,
  BooleanLiteralBlock,
  NullLiteralBlock,
  ArrayNewBlock,
  ArrayGetBlock,
  ArraySetBlock
} from './blocks.js';
import { getTheme } from './theme.ts';

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

  it('unregisters a block type', () => {
    class Custom extends Block {
      constructor(id, x, y) { super(id, x, y, 10, 10, 'c'); }
    }
    registerBlock('temp', Custom);
    unregisterBlock('temp');
    const b = createBlock('temp', '3', 0, 0, 'c');
    expect(b).toBeInstanceOf(Block);
  });

  it('provides built-in literal blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Literal/Number', NumberLiteralBlock],
      ['Literal/String', StringLiteralBlock],
      ['Literal/Boolean', BooleanLiteralBlock],
      ['Literal/Null', NullLiteralBlock]
    ];
    for (const [kind, Ctor] of cases) {
      const b = createBlock(kind, 'lit', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.w).toBe(120);
      expect(b.h).toBe(50);
      expect(b.ports).toEqual([{ id: 'out', kind: 'data', dir: 'out' }]);
      expect(b.color).toBe(theme.blockKinds.Literal);
    }
  });

  it('provides array blocks', () => {
    const theme = getTheme();
    const cases = [
      ['Array/New', ArrayNewBlock, ArrayNewBlock.ports],
      ['Array/Get', ArrayGetBlock, ArrayGetBlock.ports],
      ['Array/Set', ArraySetBlock, ArraySetBlock.ports]
    ];
    for (const [kind, Ctor, ports] of cases) {
      const b = createBlock(kind, 'arr', 0, 0, '');
      expect(b).toBeInstanceOf(Ctor);
      expect(b.ports).toEqual(ports);
      expect(b.color).toBe(theme.blockKinds.Array);
    }
  });
});

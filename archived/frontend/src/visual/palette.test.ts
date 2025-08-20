import { describe, it, expect, beforeEach } from 'vitest';
import { setRegistry, filterBlocks, PaletteBlock } from './palette.ts';

const blocks: PaletteBlock[] = [
  { kind: 'Operator/Add', name: 'Add', synonyms: ['plus'], tags: ['math'] },
  { kind: 'Log', name: 'Log', synonyms: ['print', 'debug'], tags: ['debug', 'console'] },
  { kind: 'Variable/Get', name: 'Variable Get', tags: ['variable'] }
];

describe('palette search', () => {
  beforeEach(() => setRegistry(blocks));

  it('finds by name', () => {
    const res = filterBlocks('add');
    expect(res).toHaveLength(1);
    expect(res[0].kind).toBe('Operator/Add');
  });

  it('finds by synonym', () => {
    const res = filterBlocks('print');
    expect(res).toHaveLength(1);
    expect(res[0].kind).toBe('Log');
  });

  it('finds by tag', () => {
    const res = filterBlocks('variable');
    expect(res).toHaveLength(1);
    expect(res[0].kind).toBe('Variable/Get');
  });

  it('finds by Russian translation', () => {
    const res = filterBlocks('переменная');
    expect(res).toHaveLength(1);
    expect(res[0].kind).toBe('Variable/Get');
  });

  it('returns all for empty query', () => {
    const res = filterBlocks('');
    expect(res).toHaveLength(blocks.length);
  });
});

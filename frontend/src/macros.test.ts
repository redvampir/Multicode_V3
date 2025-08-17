// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { exportMacro, insertMacro } from './macros.ts';

describe('macros import/export', () => {
  it('exports selected subgraph', () => {
    const vc: any = {
      blocksData: [
        { visual_id: 'a', kind: 'A', x: 0, y: 0 },
        { visual_id: 'b', kind: 'B', x: 10, y: 0 },
        { visual_id: 'c', kind: 'C', x: 20, y: 0 }
      ],
      connections: [
        [{ id: 'a' }, { id: 'b' }],
        [{ id: 'b' }, { id: 'c' }]
      ],
      selected: new Set(['a', 'b'])
    };
    const macro = exportMacro(vc)!;
    expect(macro.blocks.map((b: any) => b.visual_id)).toEqual(['a', 'b']);
    expect(macro.connections).toEqual([['a', 'b']]);
  });

  it('inserts macro into target graph', () => {
    const target = { blocks: [] as any[], connections: [] as [string, string][] };
    const macro = {
      blocks: [
        { visual_id: 'x', kind: 'Test', x: 0, y: 0 }
      ],
      connections: [['x', 'x'] as [string, string]]
    };
    insertMacro(target, macro);
    expect(target.blocks.length).toBe(1);
    expect(target.blocks[0].visual_id).toBe('x');
    expect(target.connections).toEqual([['x', 'x']]);
  });
});

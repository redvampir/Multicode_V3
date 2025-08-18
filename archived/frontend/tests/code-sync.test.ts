import { describe, it, expect, vi } from 'vitest';

vi.mock('../src/editor/active-block.js', () => ({
  highlightRange: { of: (v: any) => ({ value: v }) }
}));

import { registerEditor, highlightRange } from '../src/code-sync.ts';

describe('code-sync highlightRange', () => {
  it('dispatches highlight for a valid anchor', () => {
    const dispatch = vi.fn();
    registerEditor({ dispatch } as any);
    const result = highlightRange([1, 5]);
    expect(result).toBe(true);
    expect(dispatch).toHaveBeenCalledTimes(1);
    const arg = dispatch.mock.calls[0][0];
    expect(arg.selection).toEqual({ anchor: 1, head: 5 });
    expect(arg.effects[0].value).toEqual({ from: 1, to: 5 });
  });

  it('clears highlight when anchor is missing', () => {
    const dispatch = vi.fn();
    registerEditor({ dispatch } as any);
    const result = highlightRange(null);
    expect(result).toBe(false);
    expect(dispatch).toHaveBeenCalledTimes(1);
    expect(dispatch.mock.calls[0][0].effects[0].value).toBeNull();
  });
});

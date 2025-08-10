import { describe, it, expect, vi } from 'vitest';

// Mock Codemirror modules used by visual-meta.js
vi.mock('https://cdn.jsdelivr.net/npm/@codemirror/state@6.4.0/dist/index.js', () => ({
  StateField: { define: vi.fn() },
  RangeSetBuilder: class {},
}));

vi.mock('https://cdn.jsdelivr.net/npm/@codemirror/view@6.21.3/dist/index.js', () => ({
  Decoration: { mark: () => ({}) },
  EditorView: { decorations: { from: vi.fn() }, updateListener: { of: vi.fn() } },
}));

import { updateMetaComment } from '../src/editor/visual-meta.js';

describe('visual-meta synchronization', () => {
  it('reflects block coordinate changes in comments', () => {
    const original = '// @VISUAL_META {"id":"1","x":0,"y":0}\nfn main() {}';
    const view: any = {
      state: { doc: { toString: () => original } },
      dispatch: vi.fn(({ changes: { from, to, insert } }) => {
        const updated = original.slice(0, from) + insert + original.slice(to);
        view.state.doc.toString = () => updated;
      }),
    };

    const updated = updateMetaComment(view, { id: '1', x: 5, y: 7 });
    expect(updated).toBe(true);
    const text = view.state.doc.toString();
    expect(text).toContain('"x":5');
    expect(text).toContain('"y":7');
  });
});

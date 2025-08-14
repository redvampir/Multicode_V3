// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../editor/visual-meta.js', () => ({
  updateMetaComment: vi.fn()
}));
import { openBlockEditor } from './block-editor.ts';
import { updateMetaComment } from '../editor/visual-meta.js';

describe('block editor', () => {
  it('saves changes and updates meta', () => {
    const dispatch = vi.fn();
    const text = 'old';
    const metaView = {
      state: { doc: { sliceString: (f: number, t: number) => text.slice(f, t) } },
      dispatch
    } as any;

    const vc: any = {
      canvas: { getBoundingClientRect: () => ({ left: 0, top: 0 }) } as any,
      metaView,
      blockDataMap: new Map([
        ['a', { range: [0, text.length] }]
      ]),
      upsertMeta: vi.fn(),
      fileId: 'f1',
      scale: 1,
      offset: { x: 0, y: 0 }
    };

    openBlockEditor(vc, { id: 'a', x: 0, y: 0, w: 10, h: 10 });
    const textarea = document.querySelector('textarea')!;
    textarea.value = 'new';
    const btn = Array.from(document.querySelectorAll('button')).find(b => b.textContent === 'Save')!;
    btn.dispatchEvent(new Event('click'));

    expect(dispatch).toHaveBeenCalledWith({ changes: { from: 0, to: text.length, insert: 'new' } });
    expect(updateMetaComment).toHaveBeenCalled();
    expect(vc.upsertMeta).toHaveBeenCalledWith({ id: 'a' }, ['f1']);
  });
});


// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../editor/visual-meta.js', () => ({
  updateMetaComment: vi.fn()
}));
import { openBlockEditor } from './block-editor.ts';
import { updateMetaComment } from '../editor/visual-meta.js';
import { IfBlock } from './blocks.js';
import { getTheme } from './theme.ts';

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

  it('allows editing struct fields', () => {
    const json = '{"id":"a","data":{"fields":["x"]}}';
    const dispatch = vi.fn();
    const metaView = {
      state: { doc: { sliceString: () => json } },
      dispatch
    } as any;
    const vc: any = {
      canvas: { getBoundingClientRect: () => ({ left: 0, top: 0 }) } as any,
      metaView,
      blockDataMap: new Map([
        ['a', { range: [0, json.length], kind: 'Struct' }]
      ]),
      upsertMeta: vi.fn(),
      fileId: 'f1',
      scale: 1,
      offset: { x: 0, y: 0 }
    };

    openBlockEditor(vc, { id: 'a', x: 0, y: 0, w: 10, h: 10 });
    const fieldInput = document.querySelector('input')! as HTMLInputElement;
    fieldInput.value = 'y';
    const btn = Array.from(document.querySelectorAll('button')).find(b => b.textContent === 'Save')!;
    btn.dispatchEvent(new Event('click'));

    expect(dispatch).toHaveBeenCalled();
    const call = dispatch.mock.calls[0][0];
    expect(call.changes.insert).toContain('"fields":["y"]');
  });

  it('visualizes if block branches', () => {
    const theme = getTheme();
    const b = new IfBlock('if1', 0, 0);
    expect(b.ports).toEqual([
      { id: 'cond', kind: 'data', dir: 'in' },
      { id: 'exec', kind: 'exec', dir: 'in' },
      { id: 'then', kind: 'exec', dir: 'out' },
      { id: 'else', kind: 'exec', dir: 'out' }
    ]);
    expect(b.color).toBe(theme.blockKinds.If);
  });
});


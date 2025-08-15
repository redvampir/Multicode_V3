// @vitest-environment jsdom
import { describe, it, expect, vi } from 'vitest';
vi.mock('../editor/visual-meta.js', () => ({
  updateMetaComment: vi.fn()
}));
import { openBlockEditor } from './block-editor.ts';
import { updateMetaComment } from '../editor/visual-meta.js';
import { IfBlock, SwitchBlock, createBlock } from './blocks.js';
import { getTheme } from './theme.ts';
import { VisualCanvas } from './canvas.js';

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

  it('allows editing switch cases and updates ports', () => {
    const json = '{"id":"a","data":{"cases":["1"]}}';
    const dispatch = vi.fn();
    const metaView = {
      state: { doc: { sliceString: () => json } },
      dispatch
    } as any;
    const block = createBlock('Switch', 'a', 0, 0, 'Switch', undefined, { cases: ['1'] });
    const vc: any = {
      canvas: { getBoundingClientRect: () => ({ left: 0, top: 0 }) } as any,
      metaView,
      blockDataMap: new Map([
        ['a', { range: [0, json.length], kind: 'Switch', data: { cases: ['1'] } }]
      ]),
      upsertMeta: vi.fn(),
      fileId: 'f1',
      scale: 1,
      offset: { x: 0, y: 0 },
      draw: vi.fn()
    };

    openBlockEditor(vc, block);
    const caseInput = document.querySelector('input')! as HTMLInputElement;
    caseInput.value = '2';
    const btn = Array.from(document.querySelectorAll('button')).find(b => b.textContent === 'Save')!;
    btn.dispatchEvent(new Event('click'));

    expect(dispatch).toHaveBeenCalled();
    const call = dispatch.mock.calls[0][0];
    expect(call.changes.insert).toContain('"cases":["2"]');
    expect(block.ports).toEqual([
      { id: 'value', kind: 'data', dir: 'in' },
      { id: 'exec', kind: 'exec', dir: 'in' },
      { id: 'case[2]', kind: 'exec', dir: 'out' },
      { id: 'default', kind: 'exec', dir: 'out' }
    ]);
  });

  it('allows editing try exceptions', () => {
    const json = '{"id":"a","data":{"exceptions":["Error"]}}';
    const dispatch = vi.fn();
    const metaView = {
      state: { doc: { sliceString: () => json } },
      dispatch
    } as any;
    const block = createBlock('Try', 'a', 0, 0, 'Try', undefined, { exceptions: ['Error'] });
    const vc: any = {
      canvas: { getBoundingClientRect: () => ({ left: 0, top: 0 }) } as any,
      metaView,
      blockDataMap: new Map([
        ['a', { range: [0, json.length], kind: 'Try', data: { exceptions: ['Error'] } }]
      ]),
      upsertMeta: vi.fn(),
      fileId: 'f1',
      scale: 1,
      offset: { x: 0, y: 0 }
    };

    openBlockEditor(vc, block);
    const excInput = document.querySelector('input')! as HTMLInputElement;
    excInput.value = 'Exception';
    const btn = Array.from(document.querySelectorAll('button')).find(b => b.textContent === 'Save')!;
    btn.dispatchEvent(new Event('click'));

    expect(dispatch).toHaveBeenCalled();
    const call = dispatch.mock.calls[0][0];
    expect(call.changes.insert).toContain('\"exceptions\":[\"Exception\"]');
  });

  it('persists updated properties to block data map', () => {
    const json = '{"id":"a","data":{"fields":["x"]}}';
    const dispatch = vi.fn();
    const metaView = {
      state: { doc: { sliceString: () => json } },
      dispatch
    } as any;
    const dataEntry = { range: [0, json.length], kind: 'Struct', data: { fields: ['x'] } };
    const vc: any = {
      canvas: { getBoundingClientRect: () => ({ left: 0, top: 0 }) } as any,
      metaView,
      blockDataMap: new Map([[ 'a', dataEntry ]]),
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

    expect(vc.blockDataMap.get('a').data.fields).toEqual(['y']);
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

describe('groups', () => {
  function createCanvas() {
    const canvasEl = document.createElement('canvas');
    Object.defineProperty(canvasEl, 'clientWidth', { value: 200 });
    Object.defineProperty(canvasEl, 'clientHeight', { value: 200 });
    canvasEl.getContext = () => ({
      save() {},
      setTransform() {},
      clearRect() {},
      beginPath() {},
      stroke() {},
      moveTo() {},
      lineTo() {},
      fillRect() {},
      strokeRect() {},
      fillText() {},
      restore() {}
    });
    globalThis.requestAnimationFrame = () => 0;
    return canvasEl;
  }

  it('serializes grouped blocks with references', () => {
    const vc = new VisualCanvas(createCanvas());
    const a: any = { id: 'a', x: 0, y: 0, w: 10, h: 10, draw() {}, contains() { return false; } };
    const b: any = { id: 'b', x: 20, y: 0, w: 10, h: 10, draw() {}, contains() { return false; } };
    vc.blocks = [a, b];
    const da = { visual_id: 'a', kind: 'Function', x: 0, y: 0 } as any;
    const db = { visual_id: 'b', kind: 'Function', x: 20, y: 0 } as any;
    vc.blocksData = [da, db];
    vc.blockDataMap.set('a', da);
    vc.blockDataMap.set('b', db);
    vc.selected = new Set([a, b]);

    vc.groupSelected();
    const data = vc.serialize();
    expect(data.groups.length).toBe(1);
    const g = data.groups[0];
    expect(g.blocks).toEqual(['a', 'b']);
    expect(vc.blockDataMap.get('a')?.group).toBe(g.id);
    expect(vc.blockDataMap.get('b')?.group).toBe(g.id);
  });
});


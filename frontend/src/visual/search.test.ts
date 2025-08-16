// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { searchBlocks, replaceBlockLabels, createReplaceDialog } from './search.ts';

describe('visual search and replace', () => {
  it('searches by label, kind and id', () => {
    const blocks = [
      { id: 'a1', label: 'Hello', kind: 'Greeting' },
      { id: 'b2', label: 'World', kind: 'Planet' },
      { id: 'c3', label: 'Another', kind: 'Greeting' }
    ];
    expect(searchBlocks(blocks, 'hello').map(b => b.id)).toEqual(['a1']);
    expect(searchBlocks(blocks, 'kind:Greeting').map(b => b.id)).toEqual(['a1', 'c3']);
    expect(searchBlocks(blocks, 'id:b2').map(b => b.id)).toEqual(['b2']);
  });

  it('replaces labels for all matches', () => {
    const blocks = [
      { id: '1', label: 'Foo', kind: 'A' },
      { id: '2', label: 'Foo', kind: 'B' },
      { id: '3', label: 'Bar', kind: 'A' }
    ];
    const count = replaceBlockLabels(blocks, 'label:Foo', 'Baz');
    expect(count).toBe(2);
    expect(blocks.map(b => b.label)).toEqual(['Baz', 'Baz', 'Bar']);
  });

  it('modal dialog triggers replacement', () => {
    const blocks = [
      { id: '1', label: 'Old', kind: 'A' },
      { id: '2', label: 'Old', kind: 'B' }
    ];
    const dialog = createReplaceDialog((search, replace) => {
      replaceBlockLabels(blocks, search, replace);
    });
    const inputs = dialog.querySelectorAll('input');
    (inputs[0] as HTMLInputElement).value = 'Old';
    (inputs[1] as HTMLInputElement).value = 'New';
    dialog.querySelector('form')?.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
    expect(blocks[0].label).toBe('New');
    expect(blocks[1].label).toBe('New');
  });
});

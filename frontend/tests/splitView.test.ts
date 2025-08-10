/** @vitest-environment jsdom */
import { describe, it, expect, beforeEach } from 'vitest';
import { SplitManager } from '../src/layout/splitView.ts';

describe('split view', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('splits panels and synchronizes metadata', () => {
    const container = document.createElement('div');
    const manager = new SplitManager(container);
    const editor = manager.addPanel('editor');
    const canvas = manager.addPanel('canvas');

    expect(manager.panels.length).toBe(2);
    expect(container.querySelectorAll('.split-panel').length).toBe(2);

    let received: any = null;
    canvas.onMeta = (meta) => { received = meta; };

    editor.setMetadata({ '@VISUAL_META': { id: '1', x: 5 } });

    expect(received).toEqual({ '@VISUAL_META': { id: '1', x: 5 } });
    expect(canvas.metadata['@VISUAL_META']).toEqual({ id: '1', x: 5 });
  });
});

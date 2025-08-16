// @vitest-environment jsdom
import { describe, it, expect } from 'vitest';
import { openInspector } from './inspector.tsx';

describe('inspector', () => {
  it('saves changes', () => {
    const block = {
      id: 'a',
      label: 'old',
      ports: [ { id: 'p1', kind: 'data', dir: 'in' } ]
    };
    const vc: any = { blockDataMap: new Map([[ 'a', { data: { foo: 1 } } ]]) };

    openInspector(vc, block);

    const labelInput = document.getElementById('inspector-label') as HTMLInputElement;
    labelInput.value = 'new';
    const dataTextarea = document.getElementById('inspector-data') as HTMLTextAreaElement;
    dataTextarea.value = '{"foo":2}';
    const portsTextarea = document.getElementById('inspector-ports') as HTMLTextAreaElement;
    portsTextarea.value = 'p2\np3';
    const saveBtn = document.getElementById('inspector-save') as HTMLButtonElement;
    saveBtn.dispatchEvent(new Event('click'));

    expect(block.label).toBe('new');
    expect(vc.blockDataMap.get('a').data).toEqual({ foo: 2 });
    expect(block.ports.map((p: any) => p.id)).toEqual(['p2','p3']);
  });
});


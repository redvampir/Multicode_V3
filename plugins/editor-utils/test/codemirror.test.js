import { describe, it, expect } from 'vitest';
import { createEditorState } from '../codemirror.js';

describe('createEditorState', () => {
  it('uses provided doc', () => {
    const state = createEditorState({ doc: 'hello' });
    expect(state.doc.toString()).toBe('hello');
  });
  it('defaults doc when invalid', () => {
    const state = createEditorState({ doc: 123 });
    expect(state.doc.toString()).toBe('');
  });
});

import { EditorState } from '@codemirror/state';

/**
 * Initialize CodeMirror EditorState with safe defaults.
 * @param {Partial<{doc: string}>} options
 * @returns {EditorState}
 */
export function createEditorState(options = {}) {
  const doc = typeof options.doc === 'string' ? options.doc : '';
  return EditorState.create({ doc });
}

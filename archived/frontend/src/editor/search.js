import { SearchQuery } from '@codemirror/search';

/**
 * Search text inside the editor using CodeMirror's search API.
 * @param {string} query
 * @param {import('@codemirror/view').EditorView} view
 * @param {{regex?: boolean, wholeWord?: boolean}} opts
 * @returns {Array}
 */
export function searchText(query, view, opts = {}) {
  const { regex = false, wholeWord = false } = opts;
  const results = [];
  if (!query || !view) return results;
  try {
    const searchQuery = new SearchQuery({ search: query, regexp: regex, wholeWord });
    if (!searchQuery.valid) return results;
    const cursor = searchQuery.getCursor(view.state, 0, view.state.doc.length);
    while (!cursor.next().done) {
      const { from, to } = cursor.value;
      const lineInfo = view.state.doc.lineAt(from);
      results.push({
        type: 'text',
        from,
        to,
        line: lineInfo.number,
        label: lineInfo.text.trim()
      });
    }
  } catch (e) {
    console.error('Invalid search pattern', e);
  }
  return results;
}

import { scrollToMeta } from '../editor/visual-meta.js';

/**
 * Search across plain text content and blocks data.
 * @param {string} query
 * @param {import('@codemirror/view').EditorView} view
 * @param {Array} blocksData
 * @param {string} locale
 * @returns {Array}
 */
export function searchAll(query, view, blocksData, locale = 'en') {
  const results = [];
  if (!query) return results;
  const lower = query.toLowerCase();
  const fullText = view.state.doc.toString();
  const lowerText = fullText.toLowerCase();
  let idx = lowerText.indexOf(lower);
  while (idx !== -1) {
    const lineInfo = view.state.doc.lineAt(idx);
    results.push({
      type: 'text',
      from: idx,
      to: idx + query.length,
      line: lineInfo.number,
      label: lineInfo.text.trim()
    });
    idx = lowerText.indexOf(lower, idx + query.length);
  }
  if (Array.isArray(blocksData)) {
    blocksData.forEach(b => {
      const label = (b.translations && b.translations[locale]) || b.kind || '';
      if (label.toLowerCase().includes(lower)) {
        results.push({ type: 'block', id: b.visual_id, label });
      }
    });
  }
  return results;
}

/**
 * Highlight search results in editor and canvas.
 * @param {Array} results
 * @param {import('@codemirror/view').EditorView} view
 * @param {any} canvas
 */
export function highlightResults(results, view, canvas) {
  const blockIds = results.filter(r => r.type === 'block').map(r => r.id);
  if (canvas && typeof canvas.highlightBlocks === 'function') {
    canvas.highlightBlocks(blockIds);
  }
  const textRes = results.find(r => r.type === 'text');
  if (textRes) {
    view.dispatch({ selection: { anchor: textRes.from, head: textRes.to }, scrollIntoView: true });
  }
}

/**
 * Navigate to selected search result.
 * @param {object} result
 * @param {import('@codemirror/view').EditorView} view
 * @param {any} canvas
 */
export function gotoResult(result, view, canvas) {
  if (!result) return;
  if (result.type === 'text') {
    view.dispatch({ selection: { anchor: result.from, head: result.to }, scrollIntoView: true });
  } else if (result.type === 'block') {
    if (canvas && typeof canvas.selectBlock === 'function') {
      canvas.selectBlock(result.id);
    }
    scrollToMeta(result.id);
  }
}

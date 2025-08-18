import { scrollToMeta } from '../editor/visual-meta.js';
import { searchText } from '../editor/search.js';

/**
 * Search across plain text content and blocks data.
 * @param {string} query
 * @param {import('@codemirror/view').EditorView} view
 * @param {Array} blocksData
 * @param {string} locale
 * @param {object} [opts]
 * @param {boolean} [opts.regex]
 * @param {boolean} [opts.wholeWord]
 * @returns {Array}
 */
export function searchAll(query, view, blocksData, locale = 'en', opts = {}) {
  const results = [];
  if (!query) return results;

  results.push(...searchText(query, view, opts));

  if (Array.isArray(blocksData)) {
    const { regex } = opts;
    let matcher;
    if (regex) {
      try {
        matcher = new RegExp(query, 'i');
      } catch (e) {
        console.error('Invalid search pattern', e);
        return results;
      }
    } else {
      const lower = query.toLowerCase();
      matcher = { test: str => str.toLowerCase().includes(lower) };
    }
    blocksData.forEach(b => {
      const label = (b.translations && b.translations[locale]) || b.kind || '';
      if (matcher.test(label)) {
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

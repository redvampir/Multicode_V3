/**
 * Parse metadata from JSON string with validation and safe defaults.
 * @param {unknown} raw
 * @returns {{title: string, tags: string[]}}
 */
export function parseMetadata(raw) {
  const defaults = { title: '', tags: [] };
  if (typeof raw !== 'string') return defaults;
  try {
    const data = JSON.parse(raw);
    if (typeof data !== 'object' || data === null) return defaults;
    return {
      title: typeof data.title === 'string' ? data.title : '',
      tags: Array.isArray(data.tags)
        ? data.tags.filter(t => typeof t === 'string')
        : []
    };
  } catch {
    return defaults;
  }
}

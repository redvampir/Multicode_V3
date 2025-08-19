import { createTwoFilesPatch, applyPatch } from 'diff';

/**
 * Create a diff patch between two strings.
 * @param {unknown} oldStr
 * @param {unknown} newStr
 * @returns {string}
 */
export function diffStrings(oldStr, newStr) {
  const a = typeof oldStr === 'string' ? oldStr : '';
  const b = typeof newStr === 'string' ? newStr : '';
  return createTwoFilesPatch('old', 'new', a, b);
}

/**
 * Apply a patch to a string.
 * @param {unknown} original
 * @param {unknown} patch
 * @returns {string}
 */
export function applyStringPatch(original, patch) {
  const text = typeof original === 'string' ? original : '';
  const p = typeof patch === 'string' ? patch : '';
  try {
    const result = applyPatch(text, p);
    return result === false ? text : result;
  } catch {
    return text;
  }
}

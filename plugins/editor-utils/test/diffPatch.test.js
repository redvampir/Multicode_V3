import { describe, it, expect } from 'vitest';
import { diffStrings, applyStringPatch } from '../diffPatch.js';

describe('diff and patch', () => {
  it('applies generated patch', () => {
    const patch = diffStrings('hello', 'hello world');
    const result = applyStringPatch('hello', patch);
    expect(result).toBe('hello world');
  });
  it('uses safe defaults on invalid input', () => {
    const patch = diffStrings(null, null);
    const result = applyStringPatch('text', 123);
    expect(typeof patch).toBe('string');
    expect(result).toBe('text');
  });
});

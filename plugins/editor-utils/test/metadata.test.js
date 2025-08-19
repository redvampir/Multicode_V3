import { describe, it, expect } from 'vitest';
import { parseMetadata } from '../metadata.js';

describe('parseMetadata', () => {
  it('parses valid metadata', () => {
    const result = parseMetadata('{"title":"t","tags":["a","b"]}');
    expect(result).toEqual({ title: 't', tags: ['a', 'b'] });
  });
  it('returns defaults on invalid json', () => {
    const result = parseMetadata('{bad}');
    expect(result).toEqual({ title: '', tags: [] });
  });
  it('returns defaults on non-string', () => {
    const result = parseMetadata(null);
    expect(result).toEqual({ title: '', tags: [] });
  });
});

import { describe, it, expect, vi } from 'vitest';

vi.mock('@tauri-apps/api/fs', () => ({
  exists: vi.fn(),
  readTextFile: vi.fn(),
  writeTextFile: vi.fn(),
}));

import path from 'path';
import { loadVizDocument } from '../src/importer/viz.js';
import { exists, readTextFile } from '@tauri-apps/api/fs';

describe('loadVizDocument', () => {
  it('prefers .viz.json file', async () => {
    (exists as any).mockResolvedValueOnce(true);
    const doc = { nodes: [] };
    (readTextFile as any).mockResolvedValueOnce(JSON.stringify(doc));
    const result = await loadVizDocument('/proj/file.js');
    expect(result).toEqual(doc);
    expect((readTextFile as any).mock.calls[0][0]).toBe(path.join('/proj', 'file.viz.json'));
  });

  it('falls back to @viz comment', async () => {
    (exists as any).mockResolvedValueOnce(false);
    const content = '// @viz {"nodes":[1]}\nconsole.log("hi");';
    (readTextFile as any).mockResolvedValueOnce(content);
    const result = await loadVizDocument('/proj/file.js');
    expect(result).toEqual({ nodes: [1] });
  });
});

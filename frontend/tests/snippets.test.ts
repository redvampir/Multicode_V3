import { describe, it, expect } from 'vitest';
import { loadSnippets, insertSnippet, setSnippets, getSnippets, Snippet } from '../src/visual/snippets.ts';

describe('snippets', () => {
  it('loads snippets from backend', async () => {
    const snips: Snippet[] = [{ name: 'X', blocks: [], edges: [] }];
    const fetcher = async () => ({ json: async () => snips }) as any;
    const res = await loadSnippets(fetcher);
    expect(res).toEqual(snips);
    expect(getSnippets()).toEqual(snips);
  });

  it('inserts snippet into graph', () => {
    const snippet: Snippet = {
      name: 'A',
      blocks: [{ id: 'n1', kind: 'K' }],
      edges: [['n1', 'n2']]
    };
    setSnippets([snippet]);
    const graph = { blocks: [{ id: 'n2', kind: 'Q' }], edges: [] as [string, string][] };
    insertSnippet(graph, snippet);
    expect(graph.blocks).toHaveLength(2);
    expect(graph.edges).toContainEqual(['n1', 'n2']);
  });
});

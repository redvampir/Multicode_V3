export interface Snippet {
  name: string;
  /** Blocks composing the snippet graph. Only id and kind are required for tests. */
  blocks: { id: string; kind: string }[];
  /** Directed edges represented as tuples [from, to]. */
  edges: [string, string][];
}

let registry: Snippet[] = [
  {
    name: 'Sum loop',
    blocks: [
      { id: 'i', kind: 'Variable/Get' },
      { id: 'sum', kind: 'Variable/Get' },
      { id: 'add', kind: 'Operator/Add' }
    ],
    edges: [
      ['i', 'add'],
      ['sum', 'add'],
      ['add', 'sum']
    ]
  }
];

export function getSnippets(): Snippet[] {
  return [...registry];
}

export function setSnippets(snips: Snippet[]): void {
  registry = [...snips];
}

/**
 * Load snippets from backend endpoint and replace current registry.
 */
export async function loadSnippets(fetcher: typeof fetch = fetch): Promise<Snippet[]> {
  const res = await fetcher('/api/registry/snippets');
  const list: Snippet[] = await res.json();
  registry = list;
  return list;
}

/**
 * Insert snippet blocks and edges into provided graph object. The graph must
 * contain `blocks` and `edges` arrays.
 */
export function insertSnippet(target: { blocks: any[]; edges: any[] }, snippet: Snippet): void {
  for (const b of snippet.blocks) {
    target.blocks.push({ ...b });
  }
  for (const e of snippet.edges) {
    target.edges.push([...e]);
  }
}

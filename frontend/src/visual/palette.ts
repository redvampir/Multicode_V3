export interface PaletteBlock {
  kind: string;
  name: string;
  synonyms?: string[];
  tags?: string[];
}

let registry: PaletteBlock[] = [
  { kind: 'File/Read', name: 'File Read', tags: ['file', 'read'] },
  { kind: 'File/Write', name: 'File Write', tags: ['file', 'write'] }
];

/**
 * Replace current registry with provided blocks. Useful for tests.
 */
export function setRegistry(blocks: PaletteBlock[]): void {
  registry = [...blocks];
}

/**
 * Load block list from a backend registry. The registry endpoint must return
 * an array of {@link PaletteBlock} objects.
 */
export async function loadRegistry(fetcher: typeof fetch = fetch): Promise<PaletteBlock[]> {
  const res = await fetcher('/api/registry/blocks');
  const blocks: PaletteBlock[] = await res.json();
  registry = blocks;
  return blocks;
}

/**
 * Filter registered blocks by a free text query. The query is matched against
 * block name, kind, provided synonyms and tags.
 */
export function filterBlocks(query: string): PaletteBlock[] {
  const q = query.trim().toLowerCase();
  if (!q) return [...registry];
  return registry.filter(b => {
    const haystack = [b.name, b.kind, ...(b.synonyms ?? []), ...(b.tags ?? [])]
      .join(' ')
      .toLowerCase();
    return haystack.includes(q);
  });
}

/**
 * Create simple palette component inside given container element. It consists
 * of a search input and a list of blocks filtered according to the input
 * value. The list is populated from the loaded registry.
 */
export function createPalette(container: HTMLElement): void {
  const search = document.createElement('input');
  search.type = 'search';
  const list = document.createElement('ul');

  const render = () => {
    list.innerHTML = '';
    for (const block of filterBlocks(search.value)) {
      const li = document.createElement('li');
      li.textContent = block.name;
      list.appendChild(li);
    }
  };

  search.addEventListener('input', render);
  container.appendChild(search);
  container.appendChild(list);
  render();
}

export function getRegistry(): PaletteBlock[] {
  return [...registry];
}

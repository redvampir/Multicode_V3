import { getSnippets } from './snippets.ts';
import { getBlockTranslations } from '../shared/block-i18n.ts';
import { applyTranslations } from '../shared/i18n.ts';

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
let index: string[] = [];

function buildIndex(blocks: PaletteBlock[]): void {
  registry = [...blocks];
  index = blocks.map(b => {
    const translations = Object.values(getBlockTranslations(b.kind));
    return [b.name, b.kind, ...(b.synonyms ?? []), ...(b.tags ?? []), ...translations]
      .join(' ')
      .toLowerCase();
  });
}

buildIndex(registry);

/**
 * Replace current registry with provided blocks. Useful for tests.
 */
export function setRegistry(blocks: PaletteBlock[]): void {
  buildIndex(blocks);
}

/**
 * Load block list from a backend registry. The registry endpoint must return
 * an array of {@link PaletteBlock} objects.
 */
export async function loadRegistry(fetcher: typeof fetch = fetch): Promise<PaletteBlock[]> {
  const res = await fetcher('/api/registry/blocks');
  const blocks: PaletteBlock[] = await res.json();
  buildIndex(blocks);
  return blocks;
}

/**
 * Filter registered blocks by a free text query. The query is matched against
 * block name, kind, provided synonyms and tags.
 */
export function filterBlocks(query: string): PaletteBlock[] {
  const q = query.trim().toLowerCase();
  if (!q) return [...registry];
  const res: PaletteBlock[] = [];
  for (let i = 0; i < registry.length; i++) {
    if (index[i].includes(q)) res.push(registry[i]);
  }
  return res;
}

/**
 * Create simple palette component inside given container element. It consists
 * of a search input and a list of blocks filtered according to the input
 * value. Additionally, a "Snippets" tab lists predefined sub-graphs.
 */
export function createPalette(container: HTMLElement): void {
  const tabs = document.createElement('div');
  const blocksBtn = document.createElement('button');
  blocksBtn.textContent = 'Blocks';
  const snippetsBtn = document.createElement('button');
  snippetsBtn.textContent = 'Snippets';
  tabs.appendChild(blocksBtn);
  tabs.appendChild(snippetsBtn);

  // --- Blocks pane ---
  const blocksPane = document.createElement('div');
  const search = document.createElement('input');
  search.type = 'search';
  search.setAttribute('data-i18n-placeholder', 'search_placeholder');
  const list = document.createElement('ul');
  const renderBlocks = () => {
    list.innerHTML = '';
    for (const block of filterBlocks(search.value)) {
      const li = document.createElement('li');
      li.textContent = block.name;
      list.appendChild(li);
    }
  };
  search.addEventListener('input', renderBlocks);
  blocksPane.appendChild(search);
  blocksPane.appendChild(list);

  // --- Snippets pane ---
  const snippetsPane = document.createElement('div');
  const snippetList = document.createElement('ul');
  for (const snip of getSnippets()) {
    const li = document.createElement('li');
    li.textContent = snip.name;
    snippetList.appendChild(li);
  }
  snippetsPane.appendChild(snippetList);

  // --- Tab switching ---
  const showPane = (pane: 'blocks' | 'snippets') => {
    blocksPane.style.display = pane === 'blocks' ? '' : 'none';
    snippetsPane.style.display = pane === 'snippets' ? '' : 'none';
  };
  blocksBtn.addEventListener('click', () => showPane('blocks'));
  snippetsBtn.addEventListener('click', () => showPane('snippets'));

  container.appendChild(tabs);
  container.appendChild(blocksPane);
  container.appendChild(snippetsPane);

  renderBlocks();
  showPane('blocks');
  applyTranslations();
}

export function getRegistry(): PaletteBlock[] {
  return [...registry];
}

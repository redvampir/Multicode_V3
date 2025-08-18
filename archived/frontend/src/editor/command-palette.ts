/**
 * Simple command palette modal that can execute registered commands.
 * Opens with Ctrl+Shift+P and filters commands by text input.
 */

export interface Command {
  id: string;
  title: string;
  run: () => void;
}

let commands: Command[] = [];
let paletteEl: HTMLDivElement | null = null;
let inputEl: HTMLInputElement | null = null;
let listEl: HTMLUListElement | null = null;

function createPalette() {
  paletteEl = document.createElement('div');
  paletteEl.style.position = 'fixed';
  paletteEl.style.top = '0';
  paletteEl.style.left = '0';
  paletteEl.style.right = '0';
  paletteEl.style.bottom = '0';
  paletteEl.style.background = 'rgba(0,0,0,0.4)';
  paletteEl.style.display = 'none';
  paletteEl.style.zIndex = '1000';
  paletteEl.addEventListener('click', e => {
    if (e.target === paletteEl) hidePalette();
  });

  const box = document.createElement('div');
  box.style.width = '400px';
  box.style.maxHeight = '70vh';
  box.style.margin = '10vh auto';
  box.style.background = '#fff';
  box.style.borderRadius = '4px';
  box.style.boxShadow = '0 2px 10px rgba(0,0,0,0.3)';
  box.style.display = 'flex';
  box.style.flexDirection = 'column';

  inputEl = document.createElement('input');
  inputEl.type = 'text';
  inputEl.style.padding = '0.5rem';
  inputEl.style.border = 'none';
  inputEl.style.borderBottom = '1px solid #ccc';
  inputEl.style.outline = 'none';
  inputEl.addEventListener('input', () => filterCommands());
  inputEl.addEventListener('keydown', e => {
    if (e.key === 'Escape') hidePalette();
    if (e.key === 'Enter') {
      const first = listEl?.querySelector('li');
      const id = first?.getAttribute('data-id');
      if (id) executeCommand(id);
    }
  });

  listEl = document.createElement('ul');
  listEl.style.listStyle = 'none';
  listEl.style.margin = '0';
  listEl.style.padding = '0';
  listEl.style.overflowY = 'auto';
  listEl.style.flex = '1';

  box.appendChild(inputEl);
  box.appendChild(listEl);
  paletteEl.appendChild(box);
  document.body.appendChild(paletteEl);
}

function showPalette() {
  if (!paletteEl) createPalette();
  paletteEl!.style.display = 'block';
  inputEl!.value = '';
  filterCommands();
  setTimeout(() => inputEl!.focus(), 0);
}

function hidePalette() {
  if (paletteEl) paletteEl.style.display = 'none';
}

function filterCommands() {
  const query = inputEl!.value.toLowerCase();
  listEl!.innerHTML = '';
  const matches = commands.filter(c =>
    c.title.toLowerCase().includes(query) || c.id.toLowerCase().includes(query)
  );
  for (const cmd of matches) {
    const li = document.createElement('li');
    li.textContent = cmd.title;
    li.setAttribute('data-id', cmd.id);
    li.style.padding = '0.4rem 0.6rem';
    li.style.cursor = 'pointer';
    li.addEventListener('click', () => executeCommand(cmd.id));
    listEl!.appendChild(li);
  }
}

function executeCommand(id: string) {
  const cmd = commands.find(c => c.id === id);
  hidePalette();
  try {
    cmd?.run();
  } catch (e) {
    console.error('Command failed', id, e);
  }
}

export function registerCommandPalette(cmds: Command[]) {
  commands = cmds;
  document.addEventListener('keydown', e => {
    if (e.key.toUpperCase() === 'P' && e.ctrlKey && e.shiftKey) {
      e.preventDefault();
      showPalette();
    }
  });
}

export function openCommandPalette() {
  showPalette();
}


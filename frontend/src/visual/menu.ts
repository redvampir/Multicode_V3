import { hotkeys, showHotkeyHelp, zoomToFit, focusSearch } from './hotkeys';
import { exportPNG } from './export.ts';
import { emit } from '../shared/event-bus.js';
import { saveAsMacro } from '../macros.ts';

export function createSearchInput(canvas: any) {
  const input = document.createElement('input');
  input.type = 'search';
  input.placeholder = 'Search (e.g. id:foo AND tag:bar)';
  input.addEventListener('input', () => {
    canvas.search(input.value);
  });
  const hint = document.createElement('div');
  hint.textContent = 'Use field:value and combine with AND/OR';
  hint.className = 'search-hint';
  document.body.appendChild(input);
  document.body.appendChild(hint);
  return input;
}

export interface MenuItem {
  label: string;
  action?: () => void;
  shortcut?: string;
  submenu?: MenuItem[];
}

function insertFunctionTemplate(kind: 'Function/Define' | 'Function/Call' | 'Return') {
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  emit('blockCreated', { id, kind });
}

function insertSequenceTemplate() {
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  emit('blockCreated', { id, kind: 'Sequence' });
}

export const mainMenu: MenuItem[] = [
  {
    label: 'File',
    submenu: [
      { label: 'New', action: () => console.log('new file') },
      { label: 'Open', action: () => console.log('open file') },
      { label: 'Export PNG', action: exportPNG, shortcut: hotkeys.exportPNG }
    ]
  },
  {
    label: 'Edit',
    submenu: [
      { label: 'Copy Block', action: () => console.log('copy'), shortcut: hotkeys.copyBlock },
      { label: 'Paste Block', action: () => console.log('paste'), shortcut: hotkeys.pasteBlock },
      { label: 'Сгруппировать', action: () => console.log('group') },
      { label: 'Разгруппировать', action: () => console.log('ungroup') },
      { label: 'Select Connections', action: () => console.log('select'), shortcut: hotkeys.selectConnections },
      { label: 'Focus Search', action: focusSearch, shortcut: hotkeys.focusSearch }
    ]
  },
  {
    label: 'View',
    submenu: [
      { label: 'Zoom to Fit', action: zoomToFit, shortcut: hotkeys.zoomToFit }
    ]
  },
  {
    label: 'Templates',
    submenu: [
      { label: 'Function Define', action: () => insertFunctionTemplate('Function/Define') },
      { label: 'Function Call', action: () => insertFunctionTemplate('Function/Call') },
      { label: 'Return', action: () => insertFunctionTemplate('Return') },
      { label: 'Sequence', action: insertSequenceTemplate }
    ].sort((a, b) => a.label.localeCompare(b.label))
  },
  {
    label: 'Help',
    submenu: [
      { label: 'Hotkeys', action: showHotkeyHelp, shortcut: hotkeys.showHelp }
    ]
  }
];

export const contextMenus = {
  block: [
    { label: 'Copy Block', action: () => console.log('copy'), shortcut: hotkeys.copyBlock },
    { label: 'Paste Block', action: () => console.log('paste'), shortcut: hotkeys.pasteBlock },
    { label: 'Save as Macro', action: saveAsMacro },
    { label: 'Сгруппировать', action: () => console.log('group') },
    { label: 'Разгруппировать', action: () => console.log('ungroup') }
  ],
  canvas: [
    { label: 'Paste Block', action: () => console.log('paste'), shortcut: hotkeys.pasteBlock },
    { label: 'Select Connections', action: () => console.log('select'), shortcut: hotkeys.selectConnections },
    { label: 'Save as Macro', action: saveAsMacro },
    { label: 'Zoom to Fit', action: zoomToFit, shortcut: hotkeys.zoomToFit }
  ]
};


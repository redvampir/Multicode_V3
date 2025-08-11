import { hotkeys, showHotkeyHelp, zoomToFit, focusSearch } from './hotkeys';
import { exportPNG } from './canvas.js';

export function createSearchInput(canvas: any) {
  const input = document.createElement('input');
  input.type = 'search';
  input.placeholder = 'Search';
  input.addEventListener('input', () => {
    canvas.search(input.value);
  });
  document.body.appendChild(input);
  return input;
}

export interface MenuItem {
  label: string;
  action?: () => void;
  shortcut?: string;
  submenu?: MenuItem[];
}

export const mainMenu: MenuItem[] = [
  {
    label: 'File',
    submenu: [
      { label: 'New', action: () => console.log('new file') },
      { label: 'Open', action: () => console.log('open file') },
      { label: 'Экспорт в PNG', action: exportPNG }
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
    { label: 'Сгруппировать', action: () => console.log('group') },
    { label: 'Разгруппировать', action: () => console.log('ungroup') }
  ],
  canvas: [
    { label: 'Paste Block', action: () => console.log('paste'), shortcut: hotkeys.pasteBlock },
    { label: 'Select Connections', action: () => console.log('select'), shortcut: hotkeys.selectConnections },
    { label: 'Zoom to Fit', action: zoomToFit, shortcut: hotkeys.zoomToFit }
  ]
};


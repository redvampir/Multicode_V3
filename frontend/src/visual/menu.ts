import { hotkeys, showHotkeyHelp } from './hotkeys';

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
      { label: 'Open', action: () => console.log('open file') }
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
      { label: 'Focus Search', action: () => console.log('focus search'), shortcut: hotkeys.focusSearch }
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
    { label: 'Select Connections', action: () => console.log('select'), shortcut: hotkeys.selectConnections }
  ]
};


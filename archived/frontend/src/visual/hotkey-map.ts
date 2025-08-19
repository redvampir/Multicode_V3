import settings from '../../settings.json' assert { type: 'json' };

export interface HotkeyMap {
  copyBlock: string;
  pasteBlock: string;
  selectConnections: string;
  focusSearch: string;
  showHelp: string;
  openPalette: string;
  zoomToFit: string;
  undo: string;
  redo: string;
  gotoRelated: string;
  gotoLine: string;
  groupBlocks: string;
  ungroupBlocks: string;
  formatCurrentFile: string;
  insertForLoop: string;
  insertWhileLoop: string;
  insertForEachLoop: string;
  insertLogBlock: string;
  exportPNG: string;
}

const cfg: { hotkeys?: Partial<HotkeyMap> } = settings as any;

export const hotkeys: HotkeyMap = {
  copyBlock: cfg.hotkeys?.copyBlock || 'Ctrl+C',
  pasteBlock: cfg.hotkeys?.pasteBlock || 'Ctrl+V',
  selectConnections: cfg.hotkeys?.selectConnections || 'Ctrl+Shift+A',
  focusSearch: cfg.hotkeys?.focusSearch || 'Ctrl+F',
  showHelp: cfg.hotkeys?.showHelp || 'Ctrl+?',
  openPalette: cfg.hotkeys?.openPalette || 'Ctrl+P or Space Space',
  zoomToFit: cfg.hotkeys?.zoomToFit || 'Ctrl+0',
  undo: cfg.hotkeys?.undo || 'Ctrl+Z',
  redo: cfg.hotkeys?.redo || 'Ctrl+Shift+Z',
  gotoRelated: cfg.hotkeys?.gotoRelated || 'Ctrl+Alt+O',
  gotoLine: cfg.hotkeys?.gotoLine || 'Ctrl+Alt+G',
  groupBlocks: cfg.hotkeys?.groupBlocks || 'Ctrl+G',
  ungroupBlocks: cfg.hotkeys?.ungroupBlocks || 'Ctrl+Shift+G',
  formatCurrentFile: cfg.hotkeys?.formatCurrentFile || 'Alt+Shift+F',
  insertForLoop: cfg.hotkeys?.insertForLoop || 'Ctrl+Alt+F',
  insertWhileLoop: cfg.hotkeys?.insertWhileLoop || 'Ctrl+Alt+W',
  insertForEachLoop: cfg.hotkeys?.insertForEachLoop || 'Ctrl+Alt+E',
  insertLogBlock: cfg.hotkeys?.insertLogBlock || 'Ctrl+L',
  exportPNG: cfg.hotkeys?.exportPNG || 'Ctrl+Shift+E'
};

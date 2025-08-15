import settings from '../../settings.json' assert { type: 'json' };
import { createBlock } from './blocks.js';
import { getTheme } from './theme.ts';
import { createHotkeyDialog } from './hotkey-dialog.ts';
import type { VisualCanvas } from './canvas.js';
import { gotoRelated } from '../editor/navigation.js';
import { gotoLine } from '../editor/goto-line.js';
import { formatCurrentFile } from '../../scripts/format.js';
import { EditorSelection } from '@codemirror/state';
import * as commands from '@codemirror/commands';

export interface HotkeyMap {
  copyBlock: string;
  pasteBlock: string;
  selectConnections: string;
  focusSearch: string;
  showHelp: string;
  zoomToFit: string;
  undo: string;
  redo: string;
  gotoRelated: string;
  gotoLine: string;
  formatCurrentFile: string;
}

const cfg: { hotkeys?: Partial<HotkeyMap>; visual?: { gridSize?: number } } = settings as any;
const MOVE_STEP = cfg.visual?.gridSize || 10;

export const hotkeys: HotkeyMap = {
  copyBlock: cfg.hotkeys?.copyBlock || 'Ctrl+C',
  pasteBlock: cfg.hotkeys?.pasteBlock || 'Ctrl+V',
  selectConnections: cfg.hotkeys?.selectConnections || 'Ctrl+Shift+A',
  focusSearch: cfg.hotkeys?.focusSearch || 'Ctrl+F',
  showHelp: cfg.hotkeys?.showHelp || 'Ctrl+?',
  zoomToFit: cfg.hotkeys?.zoomToFit || 'Ctrl+0',
  undo: cfg.hotkeys?.undo || 'Ctrl+Z',
  redo: cfg.hotkeys?.redo || 'Ctrl+Shift+Z',
  gotoRelated: cfg.hotkeys?.gotoRelated || 'Ctrl+Alt+O',
  gotoLine: cfg.hotkeys?.gotoLine || 'Ctrl+G',
  formatCurrentFile: cfg.hotkeys?.formatCurrentFile || 'Shift+Alt+F'
};

function buildCombo(e: KeyboardEvent) {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push('Ctrl');
  if (e.altKey) parts.push('Alt');
  if (e.shiftKey) parts.push('Shift');
  const key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  parts.push(key);
  return parts.join('+');
}

export function registerHotkeys(target: Document = document) {
  target.addEventListener('keydown', handleKey);
  target.addEventListener('mousedown', handleClick);
}

export function unregisterHotkeys(target: Document = document) {
  target.removeEventListener('keydown', handleKey);
  target.removeEventListener('mousedown', handleClick);
}

function handleKey(e: KeyboardEvent) {
  const combo = buildCombo(e);
  switch (combo) {
    case hotkeys.copyBlock:
      e.preventDefault();
      copyBlock();
      break;
    case hotkeys.pasteBlock:
      e.preventDefault();
      pasteBlock();
      break;
    case hotkeys.selectConnections:
      e.preventDefault();
      selectConnections();
      break;
    case hotkeys.focusSearch:
      e.preventDefault();
      focusSearch();
      break;
    case hotkeys.showHelp:
      e.preventDefault();
      showHotkeyHelp();
      break;
    case hotkeys.zoomToFit:
      e.preventDefault();
      zoomToFit();
      break;
    case hotkeys.undo:
      e.preventDefault();
      canvasRef?.undo?.();
      break;
    case hotkeys.redo:
      e.preventDefault();
      canvasRef?.redo?.();
      break;
    case hotkeys.gotoRelated:
      e.preventDefault();
      gotoRelated((globalThis as any).view);
      break;
    case hotkeys.gotoLine:
      e.preventDefault();
      gotoLine((globalThis as any).view);
      break;
    case hotkeys.formatCurrentFile:
      e.preventDefault();
      formatCurrentFile();
      break;
    case 'F2':
      e.preventDefault();
      canvasRef?.renameSelectedBlock?.();
      break;
    case 'ArrowUp':
    case 'ArrowDown':
    case 'ArrowLeft':
    case 'ArrowRight':
      if (canvasRef?.selected?.size === 1) {
        e.preventDefault();
        const block = Array.from(canvasRef.selected)[0];
        const before = { x: block.x, y: block.y };
        switch (combo) {
          case 'ArrowUp':
            block.y -= MOVE_STEP;
            break;
          case 'ArrowDown':
            block.y += MOVE_STEP;
            break;
          case 'ArrowLeft':
            block.x -= MOVE_STEP;
            break;
          case 'ArrowRight':
            block.x += MOVE_STEP;
            break;
        }
        canvasRef.moveCallback?.(block);
        canvasRef.undoStack?.push({
          type: 'move',
          id: block.id,
          from: before,
          to: { x: block.x, y: block.y }
        });
        if (canvasRef.redoStack) canvasRef.redoStack = [];
      }
      break;
  }

  if (!e.ctrlKey && !e.altKey && !e.metaKey && e.key.length === 1) {
    keywordBuffer += e.key.toLowerCase();
    if (keywordBuffer.endsWith('var')) {
      e.preventDefault();
      insertKeywordBlock('var');
      keywordBuffer = '';
    } else if (keywordBuffer.endsWith('let')) {
      e.preventDefault();
      insertKeywordBlock('let');
      keywordBuffer = '';
    } else if (keywordBuffer.length > 3) {
      keywordBuffer = keywordBuffer.slice(-3);
    }
  }
}

function handleClick(e: MouseEvent) {
  if (!e.altKey) return;
  const view = (globalThis as any).view;
  if (!view) return;
  const pos = view.posAtCoords({ x: e.clientX, y: e.clientY });
  if (pos == null) return;
  const ranges = [...view.state.selection.ranges, EditorSelection.cursor(pos)];
  view.dispatch({ selection: EditorSelection.create(ranges, ranges.length - 1) });
  const addSel = (commands as any).addSelection;
  if (typeof addSel === 'function') addSel(view);
}

export function copyBlock() {
  if (!canvasRef || !canvasRef.selected || canvasRef.selected.size !== 1) return;
  const block = Array.from(canvasRef.selected)[0];
  const data = canvasRef.blockDataMap?.get(block.id);
  if (!data) return;
  clipboard = JSON.parse(JSON.stringify(data));
}

export function pasteBlock() {
  if (!canvasRef || !clipboard) return;
  const data = JSON.parse(JSON.stringify(clipboard));
  const theme = getTheme();
  data.visual_id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  data.x = (data.x || 0) + MOVE_STEP;
  data.y = (data.y || 0) + MOVE_STEP;
  const label = (data.translations && data.translations[canvasRef.locale]) || data.kind;
  const color = theme.blockKinds[data.kind] || theme.blockFill;
  const block = createBlock(data.kind, data.visual_id, data.x, data.y, label, color, data.data);
  canvasRef.blocks.push(block);
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(data.visual_id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}

export function selectConnections() {
  console.log('select connections');
}

export function focusSearch() {
  const el = document.querySelector('input[type="search"]') as HTMLElement | null;
  el?.focus();
}

export function showHotkeyHelp() {
  if (!hotkeyDialog) hotkeyDialog = createHotkeyDialog(hotkeys);
  hotkeyDialog.showModal();
}

let canvasRef: VisualCanvas | null = null;
let clipboard: any = null;

let hotkeyDialog: HTMLDialogElement | null = null;

let keywordBuffer = '';

export function setCanvas(vc: VisualCanvas) {
  canvasRef = vc;
}

export function zoomToFit() {
  canvasRef?.zoomToFit();
}

function insertKeywordBlock(keyword: 'var' | 'let') {
  if (!canvasRef) return;
  const theme = getTheme();
  const kind = keyword === 'var' ? 'Variable/Get' : 'Variable/Set';
  const label = keyword === 'var' ? 'Variable Get' : 'Variable Set';
  const color = theme.blockKinds.Variable || theme.blockFill;
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const block = createBlock(kind, id, 0, 0, label, color);
  canvasRef.blocks.push(block);
  const data: any = { kind, visual_id: id, x: 0, y: 0, translations: { en: label } };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}


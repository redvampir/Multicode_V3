import settings from '../../settings.json' assert { type: 'json' };
import { createBlock } from './blocks.js';
import { getTheme } from './theme.ts';

interface HotkeyMap {
  copyBlock: string;
  pasteBlock: string;
  selectConnections: string;
  focusSearch: string;
  showHelp: string;
  zoomToFit: string;
  undo: string;
  redo: string;
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
  redo: cfg.hotkeys?.redo || 'Ctrl+Shift+Z'
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
}

export function unregisterHotkeys(target: Document = document) {
  target.removeEventListener('keydown', handleKey);
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
  const block = createBlock(data.kind, data.visual_id, data.x, data.y, label, color);
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
  const list = Object.entries(hotkeys)
    .map(([name, combo]) => `${combo} - ${name}`)
    .join('\n');
  alert(list);
}

let canvasRef: any = null;
let clipboard: any = null;

export function setCanvas(vc: any) {
  canvasRef = vc;
}

export function zoomToFit() {
  canvasRef?.zoomToFit();
}


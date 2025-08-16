import settings from '../../settings.json' assert { type: 'json' };
import { GRID_SIZE } from './settings.ts';
import { createBlock } from './blocks.js';
import { getTheme } from './theme.ts';
import { createHotkeyDialog } from './hotkey-dialog.ts';
import type { VisualCanvas } from './canvas.js';
import { gotoRelated } from '../editor/navigation.js';
import { gotoLine } from '../editor/goto-line.js';
import { formatCurrentFile } from '../../scripts/format.js';
import { EditorSelection } from '@codemirror/state';
import * as commands from '@codemirror/commands';
import { openCommandPalette } from '../editor/command-palette.ts';
import { exportPNG } from './export.ts';

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
const MOVE_STEP = GRID_SIZE;

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
  formatCurrentFile: cfg.hotkeys?.formatCurrentFile || 'Shift+Alt+F',
  insertForLoop: cfg.hotkeys?.insertForLoop || 'Ctrl+Alt+F',
  insertWhileLoop: cfg.hotkeys?.insertWhileLoop || 'Ctrl+Alt+W',
  insertForEachLoop: cfg.hotkeys?.insertForEachLoop || 'Ctrl+Alt+E',
  insertLogBlock: cfg.hotkeys?.insertLogBlock || 'Ctrl+L',
  exportPNG: cfg.hotkeys?.exportPNG || 'Ctrl+Shift+E'
};

function buildCombo(e: KeyboardEvent) {
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
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
    case 'F':
      e.preventDefault();
      focusSearch();
      break;
    case hotkeys.showHelp:
      e.preventDefault();
      showHotkeyHelp();
      break;
    case 'Ctrl+P':
      e.preventDefault();
      openCommandPalette();
      break;
    case hotkeys.zoomToFit:
    case '0':
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
    case hotkeys.groupBlocks:
    case 'Meta+G':
      e.preventDefault();
      canvasRef?.groupSelected?.();
      break;
    case hotkeys.ungroupBlocks:
    case 'Meta+Shift+G':
      e.preventDefault();
      canvasRef?.ungroupSelected?.();
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
    case hotkeys.exportPNG:
      e.preventDefault();
      exportPNG();
      break;
    case hotkeys.insertForLoop:
      e.preventDefault();
      insertKeywordBlock('for');
      break;
    case hotkeys.insertWhileLoop:
      e.preventDefault();
      insertKeywordBlock('while');
      break;
    case hotkeys.insertForEachLoop:
      e.preventDefault();
      insertKeywordBlock('foreach');
      break;
    case hotkeys.insertLogBlock:
      e.preventDefault();
      insertLogBlock();
      break;
    case 'Ctrl+D':
      e.preventDefault();
      canvasRef?.copySelected?.();
      break;
    case 'Delete':
      e.preventDefault();
      canvasRef?.deleteSelected?.();
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

  if (!e.ctrlKey && !e.altKey && !e.metaKey && e.key === ' ') {
    const now = Date.now();
    if (now - lastSpaceTime < 400) {
      e.preventDefault();
      openCommandPalette();
      lastSpaceTime = 0;
    } else {
      lastSpaceTime = now;
    }
    keywordBuffer = '';
    symbolBuffer = '';
    pendingSymbol = '';
    return;
  }

  if (!e.ctrlKey && !e.altKey && !e.metaKey && e.key.length === 1) {
    if (e.key === '?') {
      pendingSymbol = '?';
      keywordBuffer = '';
      symbolBuffer = '';
    } else if (e.key === ':') {
      if (pendingSymbol === '?') {
        e.preventDefault();
        insertTernaryBlock();
      }
      pendingSymbol = '';
      keywordBuffer = '';
      symbolBuffer = '';
    } else if (e.key === '+') {
      if (pendingSymbol === '+') {
        e.preventDefault();
        insertOpBlock('++');
        pendingSymbol = '';
      } else {
        pendingSymbol = '+';
      }
      keywordBuffer = '';
      symbolBuffer = '';
    } else if (e.key === '-') {
      if (pendingSymbol === '-') {
        e.preventDefault();
        insertOpBlock('--');
        pendingSymbol = '';
      } else {
        pendingSymbol = '-';
      }
      keywordBuffer = '';
      symbolBuffer = '';
    } else if ('*/%'.includes(e.key)) {
      e.preventDefault();
      insertOperatorBlock(e.key as OperatorSymbol);
      keywordBuffer = '';
      symbolBuffer = '';
      pendingSymbol = '';
    } else if (e.key === '&' || e.key === '|') {
      symbolBuffer += e.key;
      if (symbolBuffer.endsWith('&&')) {
        e.preventDefault();
        insertLogicOperatorBlock('&&');
        keywordBuffer = '';
        symbolBuffer = '';
        pendingSymbol = '';
      } else if (symbolBuffer.endsWith('||')) {
        e.preventDefault();
        insertLogicOperatorBlock('||');
        keywordBuffer = '';
        symbolBuffer = '';
        pendingSymbol = '';
      } else if (symbolBuffer.length > 2) {
        symbolBuffer = symbolBuffer.slice(-2);
      }
    } else if ('=!<>'.includes(e.key)) {
      if (pendingSymbol) {
        if (e.key === '=' && pendingSymbol === '!') {
          e.preventDefault();
          insertComparisonOperatorBlock('!=');
          pendingSymbol = '';
        } else if (e.key === '=' && pendingSymbol === '=') {
          e.preventDefault();
          insertComparisonOperatorBlock('==');
          pendingSymbol = '';
        } else if (e.key === '=' && pendingSymbol === '>') {
          e.preventDefault();
          insertComparisonOperatorBlock('>=');
          pendingSymbol = '';
        } else if (e.key === '=' && pendingSymbol === '<') {
          e.preventDefault();
          insertComparisonOperatorBlock('<=');
          pendingSymbol = '';
        } else {
          if (pendingSymbol === '!') {
            e.preventDefault();
            insertLogicOperatorBlock('!');
          } else if (pendingSymbol === '>') {
            e.preventDefault();
            insertComparisonOperatorBlock('>');
          } else if (pendingSymbol === '<') {
            e.preventDefault();
            insertComparisonOperatorBlock('<');
          }
          pendingSymbol = e.key;
        }
      } else {
        pendingSymbol = e.key;
      }
      keywordBuffer = '';
      symbolBuffer = '';
    } else {
      if (pendingSymbol === '!') {
        e.preventDefault();
        insertLogicOperatorBlock('!');
      } else if (pendingSymbol === '>') {
        e.preventDefault();
        insertComparisonOperatorBlock('>');
      } else if (pendingSymbol === '<') {
        e.preventDefault();
        insertComparisonOperatorBlock('<');
      } else if (pendingSymbol === '+') {
        e.preventDefault();
        insertOperatorBlock('+');
      } else if (pendingSymbol === '-') {
        e.preventDefault();
        insertOperatorBlock('-');
      }
      pendingSymbol = '';
      symbolBuffer = '';
      keywordBuffer += e.key.toLowerCase();
      if (keywordBuffer.endsWith('var')) {
        e.preventDefault();
        insertKeywordBlock('var');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('let')) {
        e.preventDefault();
        insertKeywordBlock('let');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('for')) {
        e.preventDefault();
        insertKeywordBlock('for');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('while')) {
        e.preventDefault();
        insertKeywordBlock('while');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('if')) {
        e.preventDefault();
        insertKeywordBlock('if');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('switch')) {
        e.preventDefault();
        insertKeywordBlock('switch');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('return')) {
        e.preventDefault();
        insertKeywordBlock('return');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('await')) {
        e.preventDefault();
        insertKeywordBlock('await');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('delay')) {
        e.preventDefault();
        insertKeywordBlock('delay');
        keywordBuffer = '';
      } else if (keywordBuffer === 'on') {
        e.preventDefault();
        insertKeywordBlock('on');
        keywordBuffer = '';
      } else if (keywordBuffer.endsWith('concat')) {
        e.preventDefault();
        insertOperatorBlock('++');
        keywordBuffer = '';
      } else if (keywordBuffer.length > 6) {
        keywordBuffer = keywordBuffer.slice(-6);
      }
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
let symbolBuffer = '';
let pendingSymbol = '';
let lastSpaceTime = 0;

export function setCanvas(vc: VisualCanvas) {
  canvasRef = vc;
}

export function zoomToFit() {
  canvasRef?.zoomToFit();
}

function insertKeywordBlock(keyword: 'var' | 'let' | 'for' | 'while' | 'if' | 'switch' | 'return' | 'foreach' | 'await' | 'delay' | 'on') {
  if (!canvasRef) return;
  const theme = getTheme();
  let kind: string;
  let label: string;
  let color: string;
  switch (keyword) {
    case 'var':
      kind = 'Variable/Get';
      label = 'Variable Get';
      color = theme.blockKinds.Variable || theme.blockFill;
      break;
    case 'let':
      kind = 'Variable/Set';
      label = 'Variable Set';
      color = theme.blockKinds.Variable || theme.blockFill;
      break;
    case 'for':
      kind = 'Loop/For';
      label = 'For';
      color = theme.blockKinds.Loop || theme.blockFill;
      break;
    case 'while':
      kind = 'Loop/While';
      label = 'While';
      color = theme.blockKinds.Loop || theme.blockFill;
      break;
    case 'if':
      kind = 'If';
      label = 'If';
      color = theme.blockKinds.If || theme.blockFill;
      break;
    case 'switch':
      kind = 'Switch';
      label = 'Switch';
      color = theme.blockKinds.Switch || theme.blockFill;
      break;
    case 'return':
      kind = 'Return';
      label = 'Return';
      color = theme.blockKinds.Function || theme.blockFill;
      break;
    case 'foreach':
      kind = 'Loop/ForEach';
      label = 'For Each';
      color = theme.blockKinds.Loop || theme.blockFill;
      break;
    case 'await':
      kind = 'Async/Await';
      label = 'Await';
      color = theme.blockKinds.Async || theme.blockFill;
      break;
    case 'delay':
      kind = 'Async/Delay';
      label = 'Delay';
      color = theme.blockKinds.Async || theme.blockFill;
      break;
    case 'on':
      kind = 'Async/EventOn';
      label = 'Event On';
      color = theme.blockKinds.Async || theme.blockFill;
      break;
    default:
      return;
  }
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const pos = canvasRef.getFreePos ? canvasRef.getFreePos() : { x: 0, y: 0 };
  const block = createBlock(kind, id, pos.x, pos.y, label, color);
  canvasRef.blocks.push(block);
  const data: any = { kind, visual_id: id, x: pos.x, y: pos.y, translations: { en: label } };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}

type OperatorSymbol = '+' | '-' | '*' | '/' | '%' | '++';

function insertOperatorBlock(op: OperatorSymbol) {
  if (!canvasRef) return;
  const theme = getTheme();
  const mapping: Record<OperatorSymbol, { kind: string; label: string }> = {
    '+': { kind: 'Operator/Add', label: '+' },
    '-': { kind: 'Operator/Subtract', label: '-' },
    '*': { kind: 'Operator/Multiply', label: '*' },
    '/': { kind: 'Operator/Divide', label: '/' },
    '%': { kind: 'Operator/Modulo', label: '%' },
    '++': { kind: 'Operator/Concat', label: '++' }
  };
  const conf = mapping[op];
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const pos = canvasRef.getFreePos ? canvasRef.getFreePos() : { x: 0, y: 0 };
  const color = theme.blockKinds.Operator || theme.blockFill;
  const block = createBlock(conf.kind, id, pos.x, pos.y, conf.label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations: { en: conf.label }
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}

type OpSymbol = '++' | '--';

function insertOpBlock(op: OpSymbol) {
  if (!canvasRef) return;
  const theme = getTheme();
  const mapping: Record<OpSymbol, { kind: string; label: string }> = {
    '++': { kind: 'Op/Inc', label: '++' },
    '--': { kind: 'Op/Dec', label: '--' }
  };
  const conf = mapping[op];
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const pos = canvasRef.getFreePos ? canvasRef.getFreePos() : { x: 0, y: 0 };
  const color = theme.blockKinds.Operator || theme.blockFill;
  const block = createBlock(conf.kind, id, pos.x, pos.y, conf.label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations: { en: conf.label }
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}

function insertTernaryBlock() {
  if (!canvasRef) return;
  const theme = getTheme();
  const kind = 'Op/Ternary';
  const label = '?:';
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const pos = canvasRef.getFreePos ? canvasRef.getFreePos() : { x: 0, y: 0 };
  const color = theme.blockKinds.Operator || theme.blockFill;
  const block = createBlock(kind, id, pos.x, pos.y, label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations: { en: label }
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}

type LogicOperatorSymbol = '&&' | '||' | '!';

function insertLogicOperatorBlock(op: LogicOperatorSymbol) {
  if (!canvasRef) return;
  const theme = getTheme();
  const mapping: Record<LogicOperatorSymbol, { kind: string; label: string }> = {
    '&&': { kind: 'OpLogic/And', label: '&&' },
    '||': { kind: 'OpLogic/Or', label: '||' },
    '!': { kind: 'OpLogic/Not', label: '!' }
  };
  const conf = mapping[op];
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const pos = canvasRef.getFreePos ? canvasRef.getFreePos() : { x: 0, y: 0 };
  const color = theme.blockKinds.OpLogic || theme.blockFill;
  const block = createBlock(conf.kind, id, pos.x, pos.y, conf.label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations: { en: conf.label }
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}

type ComparisonOperatorSymbol = '==' | '!=' | '>' | '>=' | '<' | '<=';

function insertComparisonOperatorBlock(op: ComparisonOperatorSymbol) {
  if (!canvasRef) return;
  const theme = getTheme();
  const mapping: Record<ComparisonOperatorSymbol, { kind: string; label: string }> = {
    '==': { kind: 'OpComparison/Equal', label: '==' },
    '!=': { kind: 'OpComparison/NotEqual', label: '!=' },
    '>': { kind: 'OpComparison/Greater', label: '>' },
    '>=': { kind: 'OpComparison/GreaterEqual', label: '>=' },
    '<': { kind: 'OpComparison/Less', label: '<' },
    '<=': { kind: 'OpComparison/LessEqual', label: '<=' }
  };
  const conf = mapping[op];
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const pos = canvasRef.getFreePos ? canvasRef.getFreePos() : { x: 0, y: 0 };
  const color = theme.blockKinds.OpComparison || theme.blockFill;
  const block = createBlock(conf.kind, id, pos.x, pos.y, conf.label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations: { en: conf.label }
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
}

function insertLogBlock() {
  if (!canvasRef) return;
  const theme = getTheme();
  const kind = 'Log';
  const label = 'Log';
  const id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  const pos = canvasRef.getFreePos ? canvasRef.getFreePos() : { x: 0, y: 0 };
  const color = theme.blockKinds.Log || theme.blockFill;

  const connectFrom =
    (canvasRef as any).activeOutput ||
    (canvasRef.selected && canvasRef.selected.size === 1
      ? Array.from(canvasRef.selected)[0]
      : null);

  const block = createBlock(kind, id, pos.x, pos.y, label, color, { exec: true });
  canvasRef.blocks.push(block);
  const data: any = {
    kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations: { en: label },
    exec: true
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  if (connectFrom && typeof (canvasRef as any).connect === 'function') {
    (canvasRef as any).connect(connectFrom, block);
  }
  canvasRef.draw?.();
}


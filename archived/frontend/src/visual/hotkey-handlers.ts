import { createBlock } from './blocks.js';
import { getTheme } from './theme.ts';
import { createHotkeyDialog } from './hotkey-dialog.ts';
import type { VisualCanvas } from './canvas.js';
import { gotoRelated } from '../editor/navigation.js';
import { gotoLine } from '../editor/goto-line.js';
import { formatCurrentFile } from '../../scripts/format.js';
import { openCommandPalette } from '../editor/command-palette.ts';
import { exportPNG } from './export.ts';
import { push as pushUndo, undo as undoAction, redo as redoAction } from './undo.ts';
import { getBlockTranslations, hasTranslation, languages as blockLanguages } from '../shared/block-i18n.ts';

import { hotkeys } from './hotkey-map.ts';

let canvasRef: VisualCanvas | null = null;
let clipboard: any = null;
let hotkeyDialog: HTMLDialogElement | null = null;

export function setCanvas(vc: VisualCanvas) {
  canvasRef = vc;
}

export const editing = {
  copyBlock,
  pasteBlock,
  selectConnections,
  groupBlocks: () => canvasRef?.groupSelected?.(),
  ungroupBlocks: () => canvasRef?.ungroupSelected?.(),
  insertForLoop: () => insertKeywordBlock('for'),
  insertWhileLoop: () => insertKeywordBlock('while'),
  insertForEachLoop: () => insertKeywordBlock('foreach'),
  insertLogBlock,
  duplicateSelected: () => canvasRef?.copySelected?.(),
  deleteSelected: () => canvasRef?.deleteSelected?.(),
  renameSelected: () => canvasRef?.renameSelectedBlock?.(),
  moveSelected,
};

export const navigation = {
  focusSearch,
  showHotkeyHelp,
  openCommandPalette,
  zoomToFit: () => canvasRef?.zoomToFit(),
  gotoRelated: (view: any) => gotoRelated(view),
  gotoLine: (view: any) => gotoLine(view)
};

export const system = {
  undo: undoAction,
  redo: redoAction,
  formatCurrentFile,
  exportPNG
};

function makeTranslations(kind: string, label: string): Record<string, string> {
  if (hasTranslation(kind)) {
    return { ...getBlockTranslations(kind) };
  }
  const t: Record<string, string> = {};
  for (const l of blockLanguages()) t[l] = label;
  return t;
}

// Editing handlers
function copyBlock() {
  if (!canvasRef || !canvasRef.selected || canvasRef.selected.size !== 1) return;
  const block = Array.from(canvasRef.selected)[0];
  const data = canvasRef.blockDataMap?.get(block.id);
  if (!data) return;
  clipboard = JSON.parse(JSON.stringify(data));
}

function pasteBlock() {
  if (!canvasRef || !clipboard) return;
  const data = JSON.parse(JSON.stringify(clipboard));
  const theme = getTheme();
  data.visual_id =
    (globalThis.crypto && typeof globalThis.crypto.randomUUID === 'function')
      ? globalThis.crypto.randomUUID()
      : Math.random().toString(36).slice(2);
  data.x = (data.x || 0) + 10;
  data.y = (data.y || 0) + 10;
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

function selectConnections() {
  console.log('select connections');
}

function focusSearch() {
  const el = document.querySelector('input[type="search"]') as HTMLElement | null;
  el?.focus();
}

function showHotkeyHelp() {
  if (!hotkeyDialog) hotkeyDialog = createHotkeyDialog(hotkeys);
  hotkeyDialog.showModal();
}

export function insertKeywordBlock(keyword: 'var' | 'let' | 'for' | 'while' | 'if' | 'switch' | 'return' | 'foreach' | 'await' | 'delay' | 'on') {
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
  const translations = makeTranslations(kind, label);
  const lbl = translations[canvasRef.locale] || label;
  const block = createBlock(kind, id, pos.x, pos.y, lbl, color);
  canvasRef.blocks.push(block);
  const data: any = { kind, visual_id: id, x: pos.x, y: pos.y, translations };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
  pushUndo({
    undo: () => {
      const idx = canvasRef.blocks.indexOf(block);
      if (idx !== -1) canvasRef.blocks.splice(idx, 1);
      const dataIdx = canvasRef.blocksData.indexOf(data);
      if (dataIdx !== -1) canvasRef.blocksData.splice(dataIdx, 1);
      canvasRef.blockDataMap.delete(id);
      canvasRef.selected?.delete(block);
      canvasRef.draw?.();
    },
    redo: () => {
      canvasRef.blocks.push(block);
      canvasRef.blocksData.push(data);
      canvasRef.blockDataMap.set(id, data);
      canvasRef.selected = new Set([block]);
      canvasRef.draw?.();
    }
  });
}

type OperatorSymbol = '+' | '-' | '*' | '/' | '%' | '++';

export function insertOperatorBlock(op: OperatorSymbol) {
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
  const translations = makeTranslations(conf.kind, conf.label);
  const label = translations[canvasRef.locale] || conf.label;
  const block = createBlock(conf.kind, id, pos.x, pos.y, label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
  pushUndo({
    undo: () => {
      const idx = canvasRef.blocks.indexOf(block);
      if (idx !== -1) canvasRef.blocks.splice(idx, 1);
      const dataIdx = canvasRef.blocksData.indexOf(data);
      if (dataIdx !== -1) canvasRef.blocksData.splice(dataIdx, 1);
      canvasRef.blockDataMap.delete(id);
      canvasRef.selected?.delete(block);
      canvasRef.draw?.();
    },
    redo: () => {
      canvasRef.blocks.push(block);
      canvasRef.blocksData.push(data);
      canvasRef.blockDataMap.set(id, data);
      canvasRef.selected = new Set([block]);
      canvasRef.draw?.();
    }
  });
}

type OpSymbol = '++' | '--';

export function insertOpBlock(op: OpSymbol) {
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
  const translations = makeTranslations(conf.kind, conf.label);
  const label = translations[canvasRef.locale] || conf.label;
  const block = createBlock(conf.kind, id, pos.x, pos.y, label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
  pushUndo({
    undo: () => {
      const idx = canvasRef.blocks.indexOf(block);
      if (idx !== -1) canvasRef.blocks.splice(idx, 1);
      const dataIdx = canvasRef.blocksData.indexOf(data);
      if (dataIdx !== -1) canvasRef.blocksData.splice(dataIdx, 1);
      canvasRef.blockDataMap.delete(id);
      canvasRef.selected?.delete(block);
      canvasRef.draw?.();
    },
    redo: () => {
      canvasRef.blocks.push(block);
      canvasRef.blocksData.push(data);
      canvasRef.blockDataMap.set(id, data);
      canvasRef.selected = new Set([block]);
      canvasRef.draw?.();
    }
  });
}

export function insertTernaryBlock() {
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
  const translations = makeTranslations(kind, label);
  const lbl = translations[canvasRef.locale] || label;
  const block = createBlock(kind, id, pos.x, pos.y, lbl, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
  pushUndo({
    undo: () => {
      const idx = canvasRef.blocks.indexOf(block);
      if (idx !== -1) canvasRef.blocks.splice(idx, 1);
      const dataIdx = canvasRef.blocksData.indexOf(data);
      if (dataIdx !== -1) canvasRef.blocksData.splice(dataIdx, 1);
      canvasRef.blockDataMap.delete(id);
      canvasRef.selected?.delete(block);
      canvasRef.draw?.();
    },
    redo: () => {
      canvasRef.blocks.push(block);
      canvasRef.blocksData.push(data);
      canvasRef.blockDataMap.set(id, data);
      canvasRef.selected = new Set([block]);
      canvasRef.draw?.();
    }
  });
}

type LogicOperatorSymbol = '&&' | '||' | '!';

export function insertLogicOperatorBlock(op: LogicOperatorSymbol) {
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
  const translations = makeTranslations(conf.kind, conf.label);
  const label = translations[canvasRef.locale] || conf.label;
  const block = createBlock(conf.kind, id, pos.x, pos.y, label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
  pushUndo({
    undo: () => {
      const idx = canvasRef.blocks.indexOf(block);
      if (idx !== -1) canvasRef.blocks.splice(idx, 1);
      const dataIdx = canvasRef.blocksData.indexOf(data);
      if (dataIdx !== -1) canvasRef.blocksData.splice(dataIdx, 1);
      canvasRef.blockDataMap.delete(id);
      canvasRef.selected?.delete(block);
      canvasRef.draw?.();
    },
    redo: () => {
      canvasRef.blocks.push(block);
      canvasRef.blocksData.push(data);
      canvasRef.blockDataMap.set(id, data);
      canvasRef.selected = new Set([block]);
      canvasRef.draw?.();
    }
  });
}

type ComparisonOperatorSymbol = '==' | '!=' | '>' | '>=' | '<' | '<=';

export function insertComparisonOperatorBlock(op: ComparisonOperatorSymbol) {
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
  const translations = makeTranslations(conf.kind, conf.label);
  const label = translations[canvasRef.locale] || conf.label;
  const block = createBlock(conf.kind, id, pos.x, pos.y, label, color);
  canvasRef.blocks.push(block);
  const data: any = {
    kind: conf.kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations
  };
  canvasRef.blocksData.push(data);
  canvasRef.blockDataMap.set(id, data);
  canvasRef.selected = new Set([block]);
  canvasRef.moveCallback?.(block);
  canvasRef.draw?.();
  pushUndo({
    undo: () => {
      const idx = canvasRef.blocks.indexOf(block);
      if (idx !== -1) canvasRef.blocks.splice(idx, 1);
      const dataIdx = canvasRef.blocksData.indexOf(data);
      if (dataIdx !== -1) canvasRef.blocksData.splice(dataIdx, 1);
      canvasRef.blockDataMap.delete(id);
      canvasRef.selected?.delete(block);
      canvasRef.draw?.();
    },
    redo: () => {
      canvasRef.blocks.push(block);
      canvasRef.blocksData.push(data);
      canvasRef.blockDataMap.set(id, data);
      canvasRef.selected = new Set([block]);
      canvasRef.draw?.();
    }
  });
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

  const translations = makeTranslations(kind, label);
  const lbl = translations[canvasRef.locale] || label;
  const block = createBlock(kind, id, pos.x, pos.y, lbl, color, { exec: true });
  canvasRef.blocks.push(block);
  const data: any = {
    kind,
    visual_id: id,
    x: pos.x,
    y: pos.y,
    translations,
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
  pushUndo({
    undo: () => {
      const idx = canvasRef.blocks.indexOf(block);
      if (idx !== -1) canvasRef.blocks.splice(idx, 1);
      const dataIdx = canvasRef.blocksData.indexOf(data);
      if (dataIdx !== -1) canvasRef.blocksData.splice(dataIdx, 1);
      canvasRef.blockDataMap.delete(id);
      canvasRef.selected?.delete(block);
      canvasRef.draw?.();
    },
    redo: () => {
      canvasRef.blocks.push(block);
      canvasRef.blocksData.push(data);
      canvasRef.blockDataMap.set(id, data);
      canvasRef.selected = new Set([block]);
      canvasRef.moveCallback?.(block);
      if (connectFrom && typeof (canvasRef as any).connect === 'function') {
        (canvasRef as any).connect(connectFrom, block);
      }
      canvasRef.draw?.();
    }
  });
}

function moveSelected(dx: number, dy: number) {
  if (!canvasRef?.selected || canvasRef.selected.size !== 1) return;
  const block = Array.from(canvasRef.selected)[0];
  const before = { x: block.x, y: block.y };
  block.x += dx;
  block.y += dy;
  canvasRef.moveCallback?.(block);
  const after = { x: block.x, y: block.y };
  pushUndo({
    undo: () => {
      block.x = before.x;
      block.y = before.y;
      canvasRef.moveCallback?.(block);
      canvasRef.draw?.();
    },
    redo: () => {
      block.x = after.x;
      block.y = after.y;
      canvasRef.moveCallback?.(block);
      canvasRef.draw?.();
    }
  });
}

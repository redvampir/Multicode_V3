import { GRID_SIZE } from './settings.ts';
import { EditorSelection } from '@codemirror/state';
import * as commands from '@codemirror/commands';
import { hotkeys } from './hotkey-map.ts';
import {
  setCanvas,
  editing,
  navigation,
  system,
  insertKeywordBlock,
  insertOperatorBlock,
  insertOpBlock,
  insertTernaryBlock,
  insertLogicOperatorBlock,
  insertComparisonOperatorBlock
} from './hotkey-handlers.ts';

export { setCanvas, hotkeys };

const MOVE_STEP = GRID_SIZE;

let keywordBuffer = '';
let symbolBuffer = '';
let pendingSymbol = '';
let lastSpaceTime = 0;

const comboMap: Record<string, () => void> = {
  [hotkeys.copyBlock]: editing.copyBlock,
  [hotkeys.pasteBlock]: editing.pasteBlock,
  [hotkeys.selectConnections]: editing.selectConnections,
  [hotkeys.focusSearch]: navigation.focusSearch,
  [hotkeys.showHelp]: navigation.showHotkeyHelp,
  'Ctrl+P': navigation.openCommandPalette,
  [hotkeys.zoomToFit]: navigation.zoomToFit,
  '0': navigation.zoomToFit,
  [hotkeys.undo]: system.undo,
  [hotkeys.redo]: system.redo,
  [hotkeys.groupBlocks]: editing.groupBlocks,
  'Meta+G': editing.groupBlocks,
  [hotkeys.ungroupBlocks]: editing.ungroupBlocks,
  'Meta+Shift+G': editing.ungroupBlocks,
  [hotkeys.gotoRelated]: () => navigation.gotoRelated((globalThis as any).view),
  [hotkeys.gotoLine]: () => navigation.gotoLine((globalThis as any).view),
  [hotkeys.formatCurrentFile]: system.formatCurrentFile,
  [hotkeys.exportPNG]: system.exportPNG,
  [hotkeys.insertForLoop]: editing.insertForLoop,
  [hotkeys.insertWhileLoop]: editing.insertWhileLoop,
  [hotkeys.insertForEachLoop]: editing.insertForEachLoop,
  [hotkeys.insertLogBlock]: editing.insertLogBlock,
  'Ctrl+D': editing.duplicateSelected,
  Delete: editing.deleteSelected,
  F2: editing.renameSelected
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
  const action = comboMap[combo];
  if (action) {
    e.preventDefault();
    action();
    return;
  }

  if (combo === 'ArrowUp') {
    e.preventDefault();
    editing.moveSelected(0, -MOVE_STEP);
    return;
  }
  if (combo === 'ArrowDown') {
    e.preventDefault();
    editing.moveSelected(0, MOVE_STEP);
    return;
  }
  if (combo === 'ArrowLeft') {
    e.preventDefault();
    editing.moveSelected(-MOVE_STEP, 0);
    return;
  }
  if (combo === 'ArrowRight') {
    e.preventDefault();
    editing.moveSelected(MOVE_STEP, 0);
    return;
  }

  if (!e.ctrlKey && !e.altKey && !e.metaKey && e.key === ' ') {
    const now = Date.now();
    if (now - lastSpaceTime < 400) {
      e.preventDefault();
      navigation.openCommandPalette();
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
      insertOperatorBlock(e.key as '+' | '-' | '*' | '/' | '%');
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

export interface UndoAction {
  undo: () => void | Promise<void>;
  redo: () => void | Promise<void>;
}

const undoStack: UndoAction[] = [];
const redoStack: UndoAction[] = [];

export function push(action: UndoAction) {
  undoStack.push(action);
  redoStack.length = 0;
}

export async function undo() {
  const action = undoStack.pop();
  if (!action) return;
  await action.undo();
  redoStack.push(action);
}

export async function redo() {
  const action = redoStack.pop();
  if (!action) return;
  await action.redo();
  undoStack.push(action);
}

export function clear() {
  undoStack.length = 0;
  redoStack.length = 0;
}

export function canUndo() {
  return undoStack.length > 0;
}

export function canRedo() {
  return redoStack.length > 0;
}

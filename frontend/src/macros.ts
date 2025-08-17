import type { VisualCanvas } from './visual/canvas.js';

export interface Macro {
  blocks: any[];
  connections: [string, string][];
}

/**
 * Create a macro object from currently selected blocks and edges on the canvas.
 */
export function exportMacro(vc: VisualCanvas): Macro | null {
  const selectedIds = Array.from(vc.selected || []);
  if (!selectedIds.length) return null;
  const idSet = new Set(selectedIds);
  const blocks = vc.blocksData
    .filter(b => idSet.has(b.visual_id))
    .map(b => ({ ...b }));
  const connections: [string, string][] = [];
  for (const [from, to] of vc.connections) {
    if (idSet.has(from.id) && idSet.has(to.id)) {
      connections.push([from.id, to.id]);
    }
  }
  return { blocks, connections };
}

/**
 * Trigger download of selected subgraph as a `.macro.json` file.
 */
export function saveAsMacro(vc: VisualCanvas, name = 'macro'): void {
  const macro = exportMacro(vc);
  if (!macro) return;
  const blob = new Blob([JSON.stringify(macro, null, 2)], {
    type: 'application/json'
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `${name}.macro.json`;
  link.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

/**
 * Insert blocks and connections from a macro into the provided target object.
 * The target must contain `blocks` and `connections` arrays.
 */
export function insertMacro(
  target: { blocks: any[]; connections: [string, string][] },
  macro: Macro
): void {
  for (const b of macro.blocks) {
    target.blocks.push({ ...b });
  }
  for (const c of macro.connections) {
    target.connections.push([...c]);
  }
}

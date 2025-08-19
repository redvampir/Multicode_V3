import { getTheme } from './theme.ts';

export function getGroupId(vc, blockId) {
  for (const [id, group] of vc.groups.entries()) {
    if (group.blocks.has(blockId)) return id;
  }
  return null;
}

export function groupSelected(vc) {
  if (vc.selected.size === 0) return;
  const id = vc.nextGroupId++;
  const blocks = new Set(Array.from(vc.selected).map(b => b.id));
  const color = getTheme().blockStroke;
  const label = 'Group ' + id;
  vc.groups.set(id, { blocks, color, label });
  for (const bid of blocks) {
    const data = vc.blockDataMap.get(bid);
    if (data) data.group = id;
  }
}

export function ungroupSelected(vc) {
  const ids = new Set(Array.from(vc.selected).map(b => b.id));
  for (const [id, group] of Array.from(vc.groups.entries())) {
    let remove = false;
    for (const bid of ids) {
      if (group.blocks.has(bid)) {
        remove = true;
        break;
      }
    }
    if (remove) {
      for (const bid of group.blocks) {
        const data = vc.blockDataMap.get(bid);
        if (data) delete data.group;
      }
      vc.groups.delete(id);
    }
  }
}

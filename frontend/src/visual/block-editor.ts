import { updateMetaComment } from '../editor/visual-meta.js';
import type { EditorView } from '@codemirror/view';

export interface VisualCanvasLike {
  canvas: HTMLCanvasElement;
  metaView: EditorView | null;
  blockDataMap: Map<string, any>;
  upsertMeta(meta: any, fileIds: string[]): Promise<void> | void;
  fileId: string;
  scale: number;
  offset: { x: number; y: number };
}

export function openBlockEditor(vc: VisualCanvasLike, block: { id: string; x: number; y: number; w: number; h: number }) {
  if (!vc.metaView) return;
  const data = vc.blockDataMap.get(block.id);
  if (!data || !Array.isArray(data.range)) return;

  const rect = vc.canvas.getBoundingClientRect();
  const left = block.x * vc.scale + vc.offset.x + rect.left;
  const top = block.y * vc.scale + vc.offset.y + rect.top;
  const width = block.w * vc.scale;
  const height = block.h * vc.scale;

  const overlay = document.createElement('div');
  overlay.style.position = 'fixed';
  overlay.style.left = left + 'px';
  overlay.style.top = top + 'px';
  overlay.style.zIndex = '1000';
  overlay.style.background = '#fff';
  overlay.style.border = '1px solid #ccc';
  overlay.style.padding = '4px';

  const textarea = document.createElement('textarea');
  textarea.style.width = width + 'px';
  textarea.style.height = height + 'px';
  textarea.value = vc.metaView.state.doc.sliceString(data.range[0], data.range[1]);
  overlay.appendChild(textarea);

  const btnBar = document.createElement('div');
  btnBar.style.textAlign = 'right';
  btnBar.style.marginTop = '4px';

  const saveBtn = document.createElement('button');
  saveBtn.textContent = 'Save';
  const cancelBtn = document.createElement('button');
  cancelBtn.textContent = 'Cancel';

  btnBar.appendChild(saveBtn);
  btnBar.appendChild(cancelBtn);
  overlay.appendChild(btnBar);

  function close() {
    overlay.remove();
  }

  cancelBtn.addEventListener('click', close);
  saveBtn.addEventListener('click', () => {
    const newText = textarea.value;
    vc.metaView?.dispatch({ changes: { from: data.range[0], to: data.range[1], insert: newText } });
    updateMetaComment(vc.metaView!, { id: block.id });
    try {
      vc.upsertMeta({ id: block.id }, [vc.fileId]);
    } catch (_) {}
    close();
  });

  document.body.appendChild(overlay);
  textarea.focus();
}


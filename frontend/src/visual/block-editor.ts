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

  const fieldInputs: HTMLInputElement[] = [];
  const caseInputs: HTMLInputElement[] = [];
  const exceptionInputs: HTMLInputElement[] = [];
  if (data.kind === 'Struct') {
    let metaObj: any = {};
    try {
      metaObj = JSON.parse(textarea.value);
    } catch (_) {}
    const existing = Array.isArray(metaObj?.data?.fields) ? metaObj.data.fields : [];
    const fieldsContainer = document.createElement('div');
    fieldsContainer.style.marginTop = '4px';
    function addField(value = '') {
      const inp = document.createElement('input');
      inp.type = 'text';
      inp.value = value;
      fieldInputs.push(inp);
      fieldsContainer.appendChild(inp);
      fieldsContainer.appendChild(document.createElement('br'));
    }
    existing.forEach(f => addField(f));
    const addBtn = document.createElement('button');
    addBtn.textContent = 'Add field';
    addBtn.addEventListener('click', () => addField());
    fieldsContainer.appendChild(addBtn);
    overlay.appendChild(fieldsContainer);
  } else if (data.kind === 'Switch') {
    let metaObj: any = {};
    try {
      metaObj = JSON.parse(textarea.value);
    } catch (_) {}
    const existing = Array.isArray(metaObj?.data?.cases) ? metaObj.data.cases : [];
    const casesContainer = document.createElement('div');
    casesContainer.style.marginTop = '4px';
    function addCase(value = '') {
      const inp = document.createElement('input');
      inp.type = 'text';
      inp.value = value;
      caseInputs.push(inp);
      casesContainer.appendChild(inp);
      casesContainer.appendChild(document.createElement('br'));
    }
    existing.forEach(c => addCase(c));
    const addBtn = document.createElement('button');
    addBtn.textContent = 'Add case';
    addBtn.addEventListener('click', () => addCase());
    casesContainer.appendChild(addBtn);
    overlay.appendChild(casesContainer);
  } else if (data.kind === 'Try') {
    let metaObj: any = {};
    try {
      metaObj = JSON.parse(textarea.value);
    } catch (_) {}
    const existing = Array.isArray(metaObj?.data?.exceptions)
      ? metaObj.data.exceptions
      : [];
    const excContainer = document.createElement('div');
    excContainer.style.marginTop = '4px';
    function addExc(value = '') {
      const inp = document.createElement('input');
      inp.type = 'text';
      inp.value = value;
      exceptionInputs.push(inp);
      excContainer.appendChild(inp);
      excContainer.appendChild(document.createElement('br'));
    }
    existing.forEach(e => addExc(e));
    const addBtn = document.createElement('button');
    addBtn.textContent = 'Add exception';
    addBtn.addEventListener('click', () => addExc());
    excContainer.appendChild(addBtn);
    overlay.appendChild(excContainer);
  }

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

  textarea.addEventListener('keydown', e => {
    if (e.key === 'Escape') {
      e.preventDefault();
      close();
    }
  });

  cancelBtn.addEventListener('click', close);
  saveBtn.addEventListener('click', () => {
    let newText = textarea.value;
    if (fieldInputs.length || caseInputs.length || exceptionInputs.length) {
      let obj: any = {};
      try {
        obj = JSON.parse(newText);
      } catch (_) {}
      if (fieldInputs.length) {
        const fields = fieldInputs.map(i => i.value.trim()).filter(Boolean);
        obj.data = obj.data || {};
        obj.data.fields = fields;
      }
      if (caseInputs.length) {
        const cases = caseInputs.map(i => i.value.trim()).filter(Boolean);
        obj.data = obj.data || {};
        obj.data.cases = cases;
        (block as any).cases = cases;
        if (typeof (block as any).updatePorts === 'function') {
          (block as any).updatePorts();
          (vc as any).draw?.();
        }
      }
      if (exceptionInputs.length) {
        const exceptions = exceptionInputs.map(i => i.value.trim()).filter(Boolean);
        obj.data = obj.data || {};
        obj.data.exceptions = exceptions;
        (block as any).exceptions = exceptions;
      }
      newText = JSON.stringify(obj);
      data.data = obj.data;
    }
    vc.metaView?.dispatch({ changes: { from: data.range[0], to: data.range[1], insert: newText } });
    updateMetaComment(vc.metaView!, { id: block.id });
    try {
      vc.upsertMeta({ id: block.id }, [vc.fileId]);
    } catch (_) {}
    close();
  });

  document.body.appendChild(overlay);
  textarea.focus();
  return overlay;
}


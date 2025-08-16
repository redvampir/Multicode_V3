import type { VisualCanvasLike } from './block-editor.ts';

export interface InspectorBlock {
  id: string;
  label: string;
  ports: { id: string; kind: string; dir: string }[];
}

export function openInspector(vc: VisualCanvasLike & { blockDataMap: Map<string, any> }, block: InspectorBlock | null) {
  const existing = document.getElementById('vc-inspector');
  if (existing) existing.remove();
  if (!block) return;

  const dataEntry = vc.blockDataMap.get(block.id) || {};

  const panel = document.createElement('div');
  panel.id = 'vc-inspector';
  panel.style.position = 'fixed';
  panel.style.top = '0';
  panel.style.right = '0';
  panel.style.width = '300px';
  panel.style.height = '100%';
  panel.style.background = '#fff';
  panel.style.borderLeft = '1px solid #ccc';
  panel.style.padding = '8px';
  panel.style.overflow = 'auto';

  const lbl = document.createElement('div');
  lbl.textContent = 'Label';
  const labelInput = document.createElement('input');
  labelInput.id = 'inspector-label';
  labelInput.value = block.label || '';
  lbl.appendChild(labelInput);
  panel.appendChild(lbl);

  const dataLbl = document.createElement('div');
  dataLbl.textContent = 'Data';
  const dataTextarea = document.createElement('textarea');
  dataTextarea.id = 'inspector-data';
  try {
    dataTextarea.value = JSON.stringify(dataEntry.data || {}, null, 2);
  } catch (_) {
    dataTextarea.value = '';
  }
  dataLbl.appendChild(dataTextarea);
  panel.appendChild(dataLbl);

  const portsLbl = document.createElement('div');
  portsLbl.textContent = 'Ports';
  const portsTextarea = document.createElement('textarea');
  portsTextarea.id = 'inspector-ports';
  portsTextarea.value = (block.ports || []).map(p => p.id).join('\n');
  portsLbl.appendChild(portsTextarea);
  panel.appendChild(portsLbl);

  const btnBar = document.createElement('div');
  btnBar.style.marginTop = '8px';
  btnBar.style.textAlign = 'right';
  const saveBtn = document.createElement('button');
  saveBtn.id = 'inspector-save';
  saveBtn.textContent = 'Save';
  const cancelBtn = document.createElement('button');
  cancelBtn.id = 'inspector-cancel';
  cancelBtn.textContent = 'Cancel';
  btnBar.appendChild(saveBtn);
  btnBar.appendChild(cancelBtn);
  panel.appendChild(btnBar);

  saveBtn.addEventListener('click', () => {
    block.label = labelInput.value;
    const entry = vc.blockDataMap.get(block.id);
    if (entry) {
      try {
        entry.data = JSON.parse(dataTextarea.value || '{}');
      } catch (_) {
        entry.data = {};
      }
    }
    const ids = portsTextarea.value
      .split(/\n|,/) 
      .map(s => s.trim())
      .filter(Boolean);
    block.ports = ids.map((id, idx) => {
      const orig = (block.ports || [])[idx] || { kind: 'data', dir: 'in' };
      return { ...orig, id };
    });
    panel.remove();
  });

  cancelBtn.addEventListener('click', () => panel.remove());

  document.body.appendChild(panel);
}


import { t } from "../shared/i18n.ts";

const MAX_SNAPSHOTS = 20;
const snapshots = [];

/**
 * Add a snapshot of the current editor content.
 * @param {import('@codemirror/view').EditorView} view
 */
export function addSnapshot(view) {
  if (!view) return;
  const content = view.state.doc.toString();
  snapshots.push({ timestamp: new Date().toISOString(), content });
  if (snapshots.length > MAX_SNAPSHOTS) snapshots.shift();
}

/**
 * Show dialog to select and restore a previous snapshot.
 * @param {import('@codemirror/view').EditorView} view
 */
export function showHistory(view) {
  if (!view || snapshots.length === 0) {
    if (typeof alert === 'function') alert(t('no_history'));
    return;
  }

  const dialog = document.createElement('dialog');
  const select = document.createElement('select');
  select.size = 10;

  snapshots.forEach((snap, idx) => {
    const opt = document.createElement('option');
    opt.value = String(idx);
    opt.textContent = `${idx + 1}: ${snap.timestamp}`;
    select.appendChild(opt);
  });

  const okBtn = document.createElement('button');
  okBtn.textContent = t('restore');
  okBtn.addEventListener('click', () => {
    const idx = parseInt(select.value, 10);
    const snap = snapshots[idx];
    if (snap) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: snap.content },
      });
    }
    dialog.close();
    dialog.remove();
  });

  const cancelBtn = document.createElement('button');
  cancelBtn.textContent = t('cancel');
  cancelBtn.addEventListener('click', () => {
    dialog.close();
    dialog.remove();
  });

  dialog.appendChild(select);
  dialog.appendChild(okBtn);
  dialog.appendChild(cancelBtn);
  document.body.appendChild(dialog);
  if (typeof dialog.showModal === 'function') {
    dialog.showModal();
  } else {
    dialog.style.display = 'block';
  }
}

export { snapshots };

import { t } from "../shared/i18n.ts";

export const RELATED_EXTS = ['js', 'ts', 'jsx', 'tsx', 'json', 'html', 'css', 'md'];

/**
 * Attempt to open a file related to the current editor file.
 * Tries files with the same basename but different extensions in the
 * same directory.  If a related file is found it is opened using the
 * global `openFile` function.  Displays an alert if no related file
 * exists.
 *
 * @param {import('@codemirror/view').EditorView} view
 */
export async function gotoRelated(view) {
  const el = view?.dom;
  const current = el?.dataset?.fileId;
  if (!current) {
    alert(t('related_file_not_found'));
    return;
  }

  const slash = current.lastIndexOf('/');
  const dir = slash === -1 ? '' : current.slice(0, slash + 1);
  const file = slash === -1 ? current : current.slice(slash + 1);
  const dot = file.lastIndexOf('.');
  const name = dot === -1 ? file : file.slice(0, dot);
  const ext = dot === -1 ? '' : file.slice(dot + 1);

  for (const candidateExt of RELATED_EXTS) {
    if (candidateExt === ext) continue;
    const candidatePath = `${dir}${name}.${candidateExt}`;
    try {
      const res = await fetch(candidatePath);
      if (res.ok) {
        const text = await res.text();
        if (typeof window !== 'undefined' && typeof window.openFile === 'function') {
          await window.openFile(candidatePath, text);
          el.dataset.fileId = candidatePath;
        }
        return;
      }
    } catch (_) {
      // ignore errors and try next extension
    }
  }

  alert(t('related_file_not_found'));
}

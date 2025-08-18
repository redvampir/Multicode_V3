import { invoke } from "https://cdn.jsdelivr.net/npm/@tauri-apps/api@1.5.0/tauri.js";

const cache = new Map();

/**
 * Attach git blame tooltips to the editor line numbers.
 * @param {import('@codemirror/view').EditorView} view
 * @param {string} path File path relative to repo root
 * @returns {() => void} cleanup function
 */
export function attachGitBlame(view, path) {
  if (!view || !path) return () => {};
  const gutter = view.dom.querySelector('.cm-gutters');
  if (!gutter) return () => {};

  const tooltip = document.createElement('div');
  tooltip.style.position = 'fixed';
  tooltip.style.pointerEvents = 'none';
  tooltip.style.background = '#333';
  tooltip.style.color = '#fff';
  tooltip.style.padding = '2px 6px';
  tooltip.style.borderRadius = '4px';
  tooltip.style.fontSize = '12px';
  tooltip.style.display = 'none';
  document.body.appendChild(tooltip);

  async function loadBlame() {
    if (cache.has(path)) return cache.get(path);
    try {
      const data = await invoke('git_blame_cmd', { path });
      cache.set(path, data);
      return data;
    } catch (e) {
      console.error('git blame failed', e);
      cache.set(path, null);
      return null;
    }
  }

  let current;

  async function show(e) {
    const lineEl = e.target.closest('.cm-gutterElement');
    if (!lineEl) return;
    const line = Number(lineEl.textContent);
    if (!Number.isFinite(line)) return;
    const blame = await loadBlame();
    if (!blame) return;
    const info = blame.find(b => b.line === line);
    if (!info) return;
    const date = new Date(info.time * 1000).toLocaleDateString();
    tooltip.textContent = `${info.author} \u2013 ${date}`;
    tooltip.style.left = e.clientX + 10 + 'px';
    tooltip.style.top = e.clientY + 10 + 'px';
    tooltip.style.display = 'block';
    current = lineEl;
  }

  function move(e) {
    if (tooltip.style.display !== 'none') {
      tooltip.style.left = e.clientX + 10 + 'px';
      tooltip.style.top = e.clientY + 10 + 'px';
    }
  }

  function hide() {
    tooltip.style.display = 'none';
    current = null;
  }

  gutter.addEventListener('mouseover', show);
  gutter.addEventListener('mousemove', move);
  gutter.addEventListener('mouseout', hide);

  return () => {
    gutter.removeEventListener('mouseover', show);
    gutter.removeEventListener('mousemove', move);
    gutter.removeEventListener('mouseout', hide);
    tooltip.remove();
  };
}


import { marked } from 'https://cdn.jsdelivr.net/npm/marked@9.1.2/lib/marked.esm.js';

let active = false;
let editorEl;
let previewEl;
let currentMime = '';
let syncing = false;

function syncScroll(src, dest) {
  const ratio = src.scrollTop / Math.max(1, src.scrollHeight - src.clientHeight);
  dest.scrollTop = ratio * (dest.scrollHeight - dest.clientHeight);
}

function onEditorScroll() {
  if (syncing || !previewEl) return;
  syncing = true;
  syncScroll(editorEl, previewEl);
  syncing = false;
}

function onPreviewScroll() {
  if (syncing || !editorEl) return;
  syncing = true;
  syncScroll(previewEl, editorEl);
  syncing = false;
}

function render(view) {
  if (!active || !previewEl) return;
  const text = view.state.doc.toString();
  if (currentMime === 'text/markdown') {
    previewEl.innerHTML = marked.parse(text);
  } else {
    previewEl.innerHTML = text;
  }
}

export function enablePreview(view, mime) {
  previewEl = document.getElementById('preview');
  editorEl = view?.scrollDOM;
  currentMime = mime;
  if (!previewEl || !editorEl) return;
  previewEl.style.display = 'block';
  active = true;
  render(view);
  editorEl.addEventListener('scroll', onEditorScroll);
  previewEl.addEventListener('scroll', onPreviewScroll);
}

export function disablePreview() {
  if (editorEl) editorEl.removeEventListener('scroll', onEditorScroll);
  if (previewEl) {
    previewEl.removeEventListener('scroll', onPreviewScroll);
    previewEl.style.display = 'none';
    previewEl.innerHTML = '';
  }
  active = false;
  editorEl = null;
  previewEl = null;
  currentMime = '';
}

export function updatePreview(view) {
  if (active) render(view);
}

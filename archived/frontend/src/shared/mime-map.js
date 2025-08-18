const EXT_TO_LANG = {
  js: 'javascript',
  mjs: 'javascript',
  jsx: 'javascript',
  ts: 'javascript',
  tsx: 'javascript',
  py: 'python',
  rs: 'rust',
  html: 'html',
  htm: 'html',
  css: 'css'
};

const LANG_TO_MIME = {
  javascript: 'text/javascript',
  python: 'text/x-python',
  rust: 'text/x-rustsrc',
  html: 'text/html',
  css: 'text/css',
  plain: 'text/plain'
};

const LANG_LOADERS = {
  javascript: () => import('https://cdn.jsdelivr.net/npm/@codemirror/lang-javascript@0.19.7/dist/index.js').then(m => m.javascript()),
  python: () => import('https://cdn.jsdelivr.net/npm/@codemirror/lang-python@0.19.7/dist/index.js').then(m => m.python()),
  rust: () => import('https://cdn.jsdelivr.net/npm/@codemirror/lang-rust@0.20.2/dist/index.js').then(m => m.rust()),
  html: () => import('https://cdn.jsdelivr.net/npm/@codemirror/lang-html@0.19.7/dist/index.js').then(m => m.html()),
  css: () => import('https://cdn.jsdelivr.net/npm/@codemirror/lang-css@0.19.7/dist/index.js').then(m => m.css())
};

export function languageFromFilename(name) {
  const ext = name?.split('.').pop()?.toLowerCase();
  return EXT_TO_LANG[ext] || 'plain';
}

export function mimeFromFilename(name) {
  const lang = languageFromFilename(name);
  return LANG_TO_MIME[lang] || 'text/plain';
}

export async function loadLanguage(lang) {
  const loader = LANG_LOADERS[lang];
  return loader ? loader() : null;
}

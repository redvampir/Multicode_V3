import prettier from 'https://cdn.jsdelivr.net/npm/prettier@3.3.2/standalone.mjs';
import parserBabel from 'https://cdn.jsdelivr.net/npm/prettier@3.3.2/parser-babel.mjs';
import parserHtml from 'https://cdn.jsdelivr.net/npm/prettier@3.3.2/parser-html.mjs';
import parserTypescript from 'https://cdn.jsdelivr.net/npm/prettier@3.3.2/parser-typescript.mjs';
import settings from '../settings.json' assert { type: 'json' };

export function formatCurrentFile() {
  const view = globalThis.view;
  if (!view) return;
  const lang = globalThis.currentLang;
  let parser;
  switch (lang) {
    case 'typescript':
    case 'ts':
      parser = 'typescript';
      break;
    case 'javascript':
    case 'js':
      parser = 'babel';
      break;
    case 'html':
      parser = 'html';
      break;
    default:
      return;
  }
  const doc = view.state.doc.toString();
  try {
    const formatted = prettier.format(doc, {
      parser,
      plugins: [parserBabel, parserHtml, parserTypescript],
      ...(settings.format || {})
    });
    view.dispatch({
      changes: { from: 0, to: doc.length, insert: formatted }
    });
  } catch (err) {
    console.error('format error', err);
  }
}

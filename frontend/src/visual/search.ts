export interface SearchableBlock {
  id: string;
  label: string;
  kind: string;
  block?: { label: string };
  data?: { translations?: Record<string, string> } & Record<string, any>;
}

interface ExprCond { type: 'cond'; field: string; value: string; }
interface ExprAnd { type: 'and'; terms: Expr[]; }
interface ExprOr { type: 'or'; terms: Expr[]; }
export type Expr = ExprCond | ExprAnd | ExprOr;

function parse(input: string): Expr {
  const tokens = input.trim().split(/\s+/).filter(Boolean);
  return parseOr(tokens, 0)[0] || { type: 'and', terms: [] };
}

function parseOr(tokens: string[], pos: number): [Expr, number] {
  let [expr, i] = parseAnd(tokens, pos);
  while (i < tokens.length) {
    if (/^OR$/i.test(tokens[i])) {
      const [rhs, j] = parseAnd(tokens, i + 1);
      expr = { type: 'or', terms: [expr, rhs] };
      i = j;
    } else {
      break;
    }
  }
  return [expr, i];
}

function parseAnd(tokens: string[], pos: number): [Expr, number] {
  let [expr, i] = parseTerm(tokens, pos);
  const terms: Expr[] = [expr];
  while (i < tokens.length) {
    if (/^AND$/i.test(tokens[i])) {
      const [rhs, j] = parseTerm(tokens, i + 1);
      terms.push(rhs);
      i = j;
    } else if (/^OR$/i.test(tokens[i])) {
      break;
    } else {
      const [rhs, j] = parseTerm(tokens, i);
      terms.push(rhs);
      i = j;
    }
  }
  return terms.length === 1 ? [terms[0], i] : [{ type: 'and', terms }, i];
}

function parseTerm(tokens: string[], pos: number): [Expr, number] {
  if (pos >= tokens.length) return [{ type: 'and', terms: [] }, pos];
  const tok = tokens[pos];
  const idx = tok.indexOf(':');
  if (idx !== -1) {
    return [{ type: 'cond', field: tok.slice(0, idx), value: tok.slice(idx + 1) }, pos + 1];
  } else {
    return [{ type: 'cond', field: '*', value: tok }, pos + 1];
  }
}

function matchField(block: SearchableBlock, field: string, value: string): boolean {
  const val = value.toLowerCase();
  if (field === '*') {
    return (
      block.label.toLowerCase().includes(val) ||
      block.kind.toLowerCase().includes(val) ||
      block.id.toLowerCase().includes(val)
    );
  }
  const prop = (block as any)[field];
  return typeof prop === 'string' && prop.toLowerCase().includes(val);
}

export function matches(block: SearchableBlock, expr: Expr): boolean {
  switch (expr.type) {
    case 'and':
      return expr.terms.every(t => matches(block, t));
    case 'or':
      return expr.terms.some(t => matches(block, t));
    case 'cond':
      return matchField(block, expr.field.toLowerCase(), expr.value);
    default:
      return false;
  }
}

export function searchBlocks(blocks: SearchableBlock[], query: string): SearchableBlock[] {
  const q = query.trim();
  if (!q) return [];
  const expr = parse(q);
  return blocks.filter(b => matches(b, expr));
}

export function replaceBlockLabels(
  blocks: SearchableBlock[],
  query: string,
  replacement: string,
  locale = 'en'
): number {
  const matches = searchBlocks(blocks, query);
  for (const b of matches) {
    b.label = replacement;
    if (b.block) b.block.label = replacement;
    if (b.data) {
      if (!b.data.translations) b.data.translations = {};
      b.data.translations[locale] = replacement;
    }
  }
  return matches.length;
}

export function createReplaceDialog(
  onReplace: (search: string, replace: string) => void
): HTMLDialogElement {
  const dialog = document.createElement('dialog');
  dialog.id = 'replace-label-dialog';
  const form = document.createElement('form');
  form.method = 'dialog';

  const searchInput = document.createElement('input');
  searchInput.name = 'search';
  searchInput.placeholder = 'Search';
  const replaceInput = document.createElement('input');
  replaceInput.name = 'replace';
  replaceInput.placeholder = 'Replace';

  const submit = document.createElement('button');
  submit.type = 'submit';
  submit.textContent = 'Replace';
  const cancel = document.createElement('button');
  cancel.type = 'button';
  cancel.textContent = 'Cancel';
  cancel.addEventListener('click', () => {
    if (typeof (dialog as any).close === 'function') (dialog as any).close();
  });

  form.append(searchInput, replaceInput, submit, cancel);
  form.addEventListener('submit', e => {
    e.preventDefault();
    onReplace(searchInput.value, replaceInput.value);
    if (typeof (dialog as any).close === 'function') (dialog as any).close();
  });

  dialog.appendChild(form);
  document.body.appendChild(dialog);
  return dialog;
}

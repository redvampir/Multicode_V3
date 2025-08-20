import rawTranslations from "../../core/src/i18n/translations.json" assert { type: 'json' };

interface Translations {
  [kind: string]: { [lang: string]: string };
}

const data: Translations = rawTranslations as Translations;
const cache = new Map<string, Record<string, string>>();
const langs: string[] = Object.keys(Object.values(data)[0] || {});

function baseKind(kind: string): string {
  return kind.split('/')[0];
}

export function languages(): string[] {
  return langs;
}

export function hasTranslation(kind: string): boolean {
  return baseKind(kind) in data;
}

export function getBlockTranslations(kind: string): Record<string, string> {
  const base = baseKind(kind);
  if (cache.has(base)) {
    return cache.get(base)!;
  }
  const tr = data[base];
  if (tr) {
    cache.set(base, tr);
    return tr;
  }
  const fallback: Record<string, string> = {};
  for (const l of langs) {
    fallback[l] = base;
  }
  cache.set(base, fallback);
  return fallback;
}

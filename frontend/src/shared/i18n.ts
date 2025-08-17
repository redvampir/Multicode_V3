import en from "../locales/en.json" assert { type: 'json' };
import ru from "../locales/ru.json" assert { type: 'json' };
import es from "../locales/es.json" assert { type: 'json' };

const resources: Record<string, Record<string, string>> = {
  en: en as Record<string, string>,
  ru: ru as Record<string, string>,
  es: es as Record<string, string>
};

let current = 'en';

export function setLanguage(lang: string): void {
  if (resources[lang]) {
    current = lang;
    applyTranslations();
  }
}

export function getLanguage(): string {
  return current;
}

export function t(key: string): string {
  return resources[current][key] || key;
}

export function availableLanguages(): string[] {
  return Object.keys(resources);
}

export function getLanguageName(lang: string): string {
  return resources[lang]?.languageName || lang;
}

export function applyTranslations(): void {
  if (typeof document === 'undefined') return;
  document.documentElement.lang = current;
  document.title = t('title');
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    if (key) el.textContent = t(key);
  });
  document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
    const key = el.getAttribute('data-i18n-placeholder');
    if (key && el instanceof HTMLElement) {
      (el as HTMLInputElement).placeholder = t(key);
    }
  });
}

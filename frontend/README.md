# Frontend

## Мини-карта

Мини-карта визуального редактора поддерживает навигацию. Нажмите на любую точку мини-карты, чтобы переместить основное полотно так, чтобы выбранная область оказалась в центре.

## Internationalization

Translation files live in `src/locales`. To add a new language:

1. Create a JSON file named with the language code (e.g. `de.json`) that includes a `languageName` field and key/value pairs.
2. The language will appear in the settings dropdown automatically.
3. Set the default in `settings.json` under the `language` field.

UI strings use the `i18n.ts` helper which looks up keys and updates any element marked with `data-i18n` or `data-i18n-placeholder`.

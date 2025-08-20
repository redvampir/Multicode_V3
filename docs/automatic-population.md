# Схема автоматического пополнения

Этот документ дополняет [«Узлы памяти»](memory_nodes.md) и описывает конвейер,
который автоматически создаёт узлы из внешних источников данных. Процесс
включает четыре этапа:

## 1. Получение данных
- Используем открытые API, RSS‑ленты или публичные репозитории.
- Получаем данные через `fetch`, `curl` или `wget`.

## 2. Парсинг
- HTML/DOM: [cheerio](https://github.com/cheeriojs/cheerio) (MIT).
- CSV/JSON/YAML: встроенные парсеры Node.js или
  [`csv-parse`](https://github.com/adaltas/node-csv) (MIT) и
  [`js-yaml`](https://github.com/nodeca/js-yaml) (MIT).
- Исходный код: [tree-sitter](https://github.com/tree-sitter/tree-sitter) (Apache 2.0).

## 3. Извлечение фактов
- Лёгкие NLP‑библиотеки:
  [spaCy](https://github.com/explosion/spaCy) (MIT, модели small),
  [Natasha](https://github.com/natasha/natasha) (MIT),
  [NLTK](https://github.com/nltk/nltk) (Apache 2.0).
- Регулярные выражения и кастомные правила.

## 4. Генерация узла
- Формируем объекты узлов с `id`, `type` и произвольными атрибутами.
- Сохраняем узлы в графовую БД или локальное хранилище:
  [SQLite](https://sqlite.org) (public domain),
  [NetworkX](https://github.com/networkx/networkx) (BSD),
  [Neo4j Community Edition](https://neo4j.com/licensing/) (GPL).

## Стратегия проверки дубликатов и связывания
1. **Нормализация:** приводим имена к нижнему регистру, обрезаем пробелы,
   при необходимости транслитерируем.
2. **Уникальные ключи:** вычисляем хэши (например, SHA‑256) или составные
   идентификаторы из значимых атрибутов.
3. **Сравнение:**
   - точное — поиск ключа в реестре;
   - нестрогое — коэффициент похожести через
     [`string-similarity`](https://github.com/aceakash/string-similarity),
     [`fuzzywuzzy`](https://github.com/seatgeek/fuzzywuzzy) или расстояние
     Левенштейна.
4. **Связывание:**
   - при точном совпадении — привязываем новые данные к существующему узлу;
   - при частичном — запрашиваем ручное или полуавтоматическое подтверждение;
   - всегда фиксируем источник и версию данных для трассировки.

## Перечень используемых бесплатных технологий
- Парсинг: cheerio, csv-parse, js-yaml, tree-sitter.
- NLP: spaCy, Natasha, NLTK.
- Хранилище: SQLite, NetworkX, Neo4j Community Edition.
- Фаззи‑поиск: string-similarity, fuzzywuzzy.

Дополнительные сведения доступны в соответствующих репозиториях и лицензиях,
ссылки на которые приведены выше.


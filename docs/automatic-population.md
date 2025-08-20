# Automatic Population Workflow

This document outlines the pipeline used to automatically create nodes from
external data sources. The process consists of four stages:

## 1. Data acquisition
- Source open APIs, RSS feeds, or public repositories.
- Retrieve data with `fetch`, `curl`, or `wget`.

## 2. Parsing
- HTML/DOM: [cheerio](https://github.com/cheeriojs/cheerio) (MIT).
- CSV/JSON/YAML: built‑in Node.js parsers or
  [`csv-parse`](https://github.com/adaltas/node-csv) (MIT) and
  [`js-yaml`](https://github.com/nodeca/js-yaml) (MIT).
- Source code: [tree-sitter](https://github.com/tree-sitter/tree-sitter) (Apache 2.0).

## 3. Fact extraction
- Lightweight NLP libraries such as
  [spaCy](https://github.com/explosion/spaCy) (MIT, small models),
  [Natasha](https://github.com/natasha/natasha) (MIT),
  [NLTK](https://github.com/nltk/nltk) (Apache 2.0).
- Custom rules and regular expressions.

## 4. Node generation
- Assemble node objects with `id`, `type`, and arbitrary attributes.
- Persist nodes in a graph database or local store:
  [SQLite](https://sqlite.org) (public domain),
  [NetworkX](https://github.com/networkx/networkx) (BSD), or
  [Neo4j Community Edition](https://neo4j.com/licensing/) (GPL).

## Deduplication and linking strategy
1. **Normalization:** convert names to lowercase, trim whitespace,
   and apply transliteration.
2. **Unique keys:** compute hashes (e.g. SHA‑256) or derive composite
   identifiers from meaningful attributes.
3. **Comparison:**
   - exact: look up the key in the registry;
   - fuzzy: apply similarity metrics such as
     [`string-similarity`](https://github.com/aceakash/string-similarity),
     [`fuzzywuzzy`](https://github.com/seatgeek/fuzzywuzzy), or Levenshtein
     distance.
4. **Linking:**
   - on exact match, attach new data to the existing node;
   - on partial match, request manual or semi‑automatic confirmation;
   - always record data source and version for traceability.

## Summary of free technologies
- Parsing: cheerio, csv-parse, js-yaml, tree-sitter.
- NLP: spaCy, Natasha, NLTK.
- Storage: SQLite, NetworkX, Neo4j Community Edition.
- Fuzzy matching: string-similarity, fuzzywuzzy.

For more information, consult the respective repositories and licenses linked above.

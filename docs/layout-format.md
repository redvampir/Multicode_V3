# Layout export format

## Навигация
- [Обзор Нейры](neira/README.md)
- [Узлы действий](neira/action-nodes.md)
- [Узлы анализа](neira/analysis-nodes.md)
- [Узлы памяти](neira/memory-nodes.md)
- [Архитектура анализа](neira/analysis-architecture.md)
- [Поддерживающие системы](neira/support-systems.md)
- [Личность Нейры](neira/personality.md)
- [Шаблон узла](neira/node-template.md)
- [Политика источников](neira/source-policy.md)

## Оглавление
- [JSON structure](#json-structure)
- [Example](#example)

The visual editor can save and restore its state through JSON. Use the **Export** and **Import** buttons in the interface or call the methods directly on `VisualCanvas`.

## JSON structure

```json
{
  "blocks": [],
  "connections": [["a", "b"]],
  "offset": { "x": 0, "y": 0 },
  "scale": 1
}
```

- `blocks` — array of block metadata (`blocksData`).
- `connections` — list of edges as pairs of block `visual_id`s.
- `offset` — current pan offset of the canvas.
- `scale` — zoom level.

## Example

```js
// Export current layout
const json = JSON.stringify(vc.serialize(), null, 2);

// Restore layout
vc.load(JSON.parse(json));
```

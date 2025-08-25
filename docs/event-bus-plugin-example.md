# Plugin reacting to events

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
- [Plugin reacting to events](#plugin-reacting-to-events)

Plugins can subscribe to the shared event bus (`frontend/src/shared/event-bus.js`) to react to editor actions.

```js
import { on } from '../src/shared/event-bus.js';

export default function activate() {
  on('blockSelected', ({ id }) => {
    console.log('Selected block:', id);
  });

  on('metaUpdated', meta => {
    console.log('Meta updated:', meta);
  });
}
```

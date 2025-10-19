# Плагин, реагирующий на события

Плагины могут подписываться на общую шину событий (`frontend/src/shared/event-bus.js`), чтобы реагировать на действия редактора.

Определения терминов см. в [glossary.md](glossary.md).

```js
import { on } from "../src/shared/event-bus.js";

export default function activate() {
  on("blockSelected", ({ id }) => {
    console.log("Выбран блок:", id);
  });

  on("metaUpdated", (meta) => {
    console.log("Метаданные обновлены:", meta);
  });
}
```

Общая информация по созданию плагинов: [plugin-guide.md](plugin-guide.md).

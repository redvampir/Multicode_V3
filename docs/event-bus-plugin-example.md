# Plugin reacting to events

Plugins can subscribe to the shared event bus (`frontend/src/shared/event-bus.js`) to react to editor actions.

See [glossary.md](glossary.md) for terminology definitions.

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

See [plugin-guide.md](plugin-guide.md) for general information on writing plugins.

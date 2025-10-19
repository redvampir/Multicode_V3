# Плагины

В проекте поддерживается расширение функциональности через плагины.
Плагин состоит из двух частей:

- **Backend** – реализует трейт [`Plugin`](../../backend/src/plugins/mod.rs) и
  сообщает о дополнительных типах блоков.
- **Frontend** – предоставляет визуальный компонент для нового блока и
  регистрирует его через функцию `registerBlock`, а при необходимости
  удаляет через `unregisterBlock`.

## Backend API

```rust
use backend::plugins::{Plugin, BlockDescriptor};

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &'static str { "my-plugin" }

    fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }

    fn blocks(&self) -> Vec<BlockDescriptor> {
        vec![BlockDescriptor {
            kind: "MyBlock".into(),
            label: Some("Мой блок".into()),
            version: env!("CARGO_PKG_VERSION").into(),
        }]
    }
}
```

## Frontend пример

```javascript
// my-block.js
export function register({ Block, registerBlock }) {
  class MyBlock extends Block {
    constructor(id, x, y, w, h, label, color, extras = {}) {
      super(id, x, y, w, h, label, color);
      this.extras = extras;
    }

    draw(ctx) {
      super.draw(ctx);
      ctx.strokeStyle = this.extras.outline || "red";
      ctx.strokeRect(this.x, this.y, this.w, this.h);
    }
  }

  registerBlock("MyBlock", MyBlock);
}
```

Загрузить модуль можно, передав путь в `loadBlockPlugins`:

```javascript
await loadBlockPlugins(["./my-block.js"]);
```

При обновлении кода плагина его можно перезагрузить без перезагрузки страницы:

```javascript
await reloadPlugins(["./my-block.js"]);
```

Если блок больше не нужен, его можно удалить из реестра:

```javascript
unregisterBlock("MyBlock");
```

Поле `extras` из структуры [`VisualMeta`](../../backend/src/meta/mod.rs) позволяет
передавать в блок произвольные данные плагина. В примере выше цвет обводки
блока можно настроить с помощью метаданных:

```text
// @VISUAL_META {"id":"1","x":0,"y":0,"extras":{"outline":"blue"}}
```

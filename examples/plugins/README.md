# Плагины

В проекте поддерживается расширение функциональности через плагины.
Плагин состоит из двух частей:

* **Backend** – реализует трейт [`Plugin`](../../backend/src/plugins/mod.rs) и
  сообщает о дополнительных типах блоков.
* **Frontend** – предоставляет визуальный компонент для нового блока и
  регистрирует его через функцию `registerBlock`.

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
    draw(ctx) {
      super.draw(ctx);
      ctx.strokeStyle = 'red';
      ctx.strokeRect(this.x, this.y, this.w, this.h);
    }
  }

  registerBlock('MyBlock', MyBlock);
}
```

Загрузить модуль можно, передав путь в `loadBlockPlugins`:

```javascript
await loadBlockPlugins(['./my-block.js']);
```

# Руководство по созданию плагина

Этот пример показывает пошаговое создание плагина, добавляющего новый блок `ExampleBlock`.

Термины см. в [глоссарии](glossary.md).

## 1. Backend

Создайте библиотеку и реализуйте трейт [`Plugin`](plugins.md#backend-api):

```rust
use backend::plugins::{BlockDescriptor, Plugin};

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &'static str { "example-plugin" }

    fn version(&self) -> &str { env!("CARGO_PKG_VERSION") }

    fn blocks(&self) -> Vec<BlockDescriptor> {
        vec![BlockDescriptor {
            kind: "ExampleBlock".into(),
            label: Some("Example block".into()),
            version: env!("CARGO_PKG_VERSION").into(),
        }]
    }
}
```

Соберите библиотеку в WebAssembly:

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib"]
```

```bash
cargo build --release --target wasm32-unknown-unknown
```

Полученный файл можно загрузить через `WasmPlugin::from_file`.

## 2. Frontend

Опишите визуальный компонент для блока и зарегистрируйте его:

```ts
export function register({ Block, registerBlock }: any) {
  class ExampleBlock extends Block {
    constructor(id: string, x: number, y: number) {
      super(id, x, y, 120, 50, "Example");
    }
  }
  registerBlock("ExampleBlock", ExampleBlock);
}
```

## 3. Загрузка

Backend сообщает клиенту о новых блоках через `BlockDescriptor`, после чего
frontend может вызвать `loadPlugins` и передать путь к модулю, описанному выше.

## Отличия от старого фронтенда

- Ранее плагины были чистыми JavaScript‑модулями и автоматически подхватывались
  загрузчиком `plugin-loader.ts`.
- Теперь основная часть плагина выполняется на backend и может быть собрана в
  WebAssembly. Frontend лишь подключает визуальные компоненты согласно
  описаниям от backend.

Дополнительные сведения см. в [plugins.md](plugins.md).

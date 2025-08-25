# Синхронизация редакторов

`SyncEngine` обеспечивает обмен данными между текстовым и визуальным редакторами.

## Потоки данных

1. **Изменение текста**
   Текстовый редактор отправляет `SyncMessage::TextChanged` с новым содержимым и языком.
   `SyncEngine` извлекает метаданные и возвращает их вместе с обновлённым кодом.
   Визуальный редактор использует метаданные для обновления схемы.

2. **Изменение блок-схемы**  
   Визуальный редактор отправляет `SyncMessage::VisualChanged` с обновлёнными метаданными.  
   `SyncEngine` встраивает метаданные в текст и возвращает новое содержимое.  
   Текстовый редактор обновляет текст.

## Пример

```rust
use desktop::sync::{SyncEngine, SyncMessage};
use multicode_core::parser::Lang;

let mut engine = SyncEngine::new(Lang::Rust);
let (_code, metas) = engine
    .handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust))
    .unwrap();
// передать metas визуальному редактору
```

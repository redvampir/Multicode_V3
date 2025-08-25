# Синхронизация редакторов

`SyncEngine` обеспечивает обмен данными между текстовым и визуальным редакторами.

## ASTParser

`ASTParser` разбирает исходный код в абстрактное синтаксическое дерево и сопоставляет узлы
с визуальными метаданными. Для корректного разбора необходимо передавать язык `Lang`,
что особенно важно с появлением новых поддерживаемых языков: C, C++, Java и C#.

## Формат SyntaxTree

`SyntaxTree` представляет собой плоский список узлов `SyntaxNode`. Каждый узел содержит
структуру `Block` с идентификатором `visual_id`, уникально связывающим его с записью
`VisualMeta`. Поле `visual_id` сохраняется при повторных разборах и позволяет
соотносить элементы текста и блок-схемы.

## Обновление визуального блока на основе AST

```rust
use desktop::sync::ASTParser;
use multicode_core::parser::Lang;

let mut parser = ASTParser::new(Lang::Rust);
let tree = parser.parse("fn main() {}", &metas);
for node in tree.nodes {
    if node.block.visual_id == "0" {
        // переместить блок или обновить его метаданные
    }
}
```

## Потоки данных

1. **Изменение текста**
   Текстовый редактор отправляет `SyncMessage::TextChanged` с новым содержимым и языком.
   `SyncEngine` извлекает метаданные и возвращает их вместе с обновлённым кодом.
   Визуальный редактор использует метаданные для обновления схемы.

2. **Изменение блок-схемы**
   Визуальный редактор отправляет `SyncMessage::VisualChanged` с обновлёнными метаданными.
   `SyncEngine` встраивает метаданные в текст и возвращает новое содержимое.
   Текстовый редактор обновляет текст.

## Сопоставление кода и метаданных

`SyncEngine` и вспомогательный `ElementMapper` позволяют определить связь между
позициями в исходном тексте и идентификаторами блоков. Метод
[`id_at`](../desktop/src/sync/engine.rs) возвращает идентификатор по байтовому
смещению, [`id_at_position`](../desktop/src/sync/engine.rs) — по номеру строки и
столбца, а [`range_of`](../desktop/src/sync/engine.rs) выдаёт диапазон кода по
идентификатору. Во время разбора фиксируются две дополнительные ситуации:

- `orphaned_blocks` — метаданные, для которых не найден соответствующий фрагмент
  кода;
- `unmapped_code` — участки кода без метаданных.

Эти списки доступны через одноимённые методы `SyncEngine` и могут использоваться
для диагностики несоответствий между текстом и визуальным представлением.

## Разрешение конфликтов

При одновременном редактировании текста и блок-схемы версии `VisualMeta` могут
расходиться. `ConflictResolver` сравнивает две версии и определяет тип
конфликта:

- `Structural` — различия в переводах, `extends` или `origin`.
- `Movement` — изменение координат блока на холсте (`x`, `y`).
- `MetaComment` — теги, ссылки, якоря, тесты, заметки ИИ или дополнительные
  поля (`extras`).

Для каждого конфликта выбирается стратегия `ResolutionOption`:

- `Text` — принять текстовое представление.
- `Visual` — принять визуальную версию.
- `Merge` — объединить данные (используется, например, для тегов).

Глобальная политика `ResolutionPolicy` (`PreferText` или `PreferVisual`)
определяет приоритет в случае структурных расхождений.

### Пример: расхождение версий

```rust
use desktop::sync::{ResolutionPolicy, SyncEngine, SyncMessage};
use multicode_core::parser::Lang;

let mut engine = SyncEngine::new(Lang::Rust, ResolutionPolicy::PreferText);
let code = r#"// @VISUAL_META {\"id\":\"1\",\"x\":0.0,\"y\":0.0}\nfn main() {}"#;
let (_code, mut metas) =
    engine.handle(SyncMessage::TextChanged(code.into(), Lang::Rust)).unwrap();

// визуальный редактор переместил блок и увеличил версию
let mut meta = metas.pop().unwrap();
meta.version += 1;
meta.x = 10.0;

let (_code, _metas) = engine.handle(SyncMessage::VisualChanged(meta)).unwrap();
// координаты будут взяты из визуального представления
```

## Пример

```rust
use desktop::sync::{ResolutionPolicy, SyncEngine, SyncMessage};
use multicode_core::parser::Lang;

let mut engine = SyncEngine::new(Lang::Rust, ResolutionPolicy::PreferText);
let (_code, metas) = engine
    .handle(SyncMessage::TextChanged("fn main() {}".into(), Lang::Rust))
    .unwrap();
// передать metas визуальному редактору
```

## Генерация и форматирование кода

`CodeGenerator` восстанавливает исходный текст из набора `VisualMeta` и
соответствующих узлов `Block`. После генерации полученную строку можно выровнять
и добавить отступы с помощью `format_generated_code`.

### Пример

```rust
use desktop::sync::{CodeGenerator, format_generated_code, FormattingStyle};
use multicode_core::meta::VisualMeta;
use multicode_core::parser::{Block, Lang};
use chrono::Utc;
use std::collections::HashMap;

let mut translations = HashMap::new();
translations.insert("rust".into(), "fn main() {}".into());

let metas = vec![VisualMeta {
    version: 1,
    id: "1".into(),
    x: 0.0,
    y: 0.0,
    tags: vec![],
    links: vec![],
    anchors: vec![],
    tests: vec![],
    extends: None,
    origin: None,
    translations,
    ai: None,
    extras: None,
    updated_at: Utc::now(),
}];

let blocks = vec![Block {
    visual_id: "1".into(),
    node_id: 0,
    kind: String::new(),
    range: 0..0,
    anchors: vec![],
}];

let gen = CodeGenerator::new(Lang::Rust, true);
let code = gen.generate(&metas, &blocks).unwrap();
let formatted = format_generated_code(&code, 0, FormattingStyle::Spaces, 4);
assert_eq!(formatted.trim(), "fn main() {}");
```

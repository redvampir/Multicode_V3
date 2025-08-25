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

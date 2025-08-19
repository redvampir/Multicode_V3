# Архитектура анализа

Документ описывает API узлов анализа, иерархию их подтипов и варианты расширения системы на Rust.

## API

`AnalysisNode` определяет базовый контракт для всех узлов.

```rust
pub trait AnalysisNode {
    fn id(&self) -> &str;
    fn node_type(&self) -> &str;
    fn analyze(&self, input: &str) -> AnalysisResult;
    fn explain(&self) -> String;
}
```

Тип `AnalysisResult` содержит произвольные метрики и текстовое объяснение.

```rust
pub struct AnalysisResult {
    pub metrics: HashMap<String, f32>,
    pub explanation: String,
}
```

## Иерархия подтипов

```text
AnalysisNode
├─ DataSourceNode
├─ ReasoningNode
└─ DomainNode
   ├─ ProgrammingSyntaxNode
   ├─ NaturalLanguageNode
   └─ DomainSpecificNode
```

- **DataSourceNode** — адаптеры к внешним источникам данных.
- **ReasoningNode** — агрегируют и интерпретируют результаты анализа.
- **DomainNode** — специализированные подтипы для конкретных областей знаний.

## Примеры расширений на Rust

```rust
use std::collections::HashMap;

pub struct ComplexityNode;

impl AnalysisNode for ComplexityNode {
    fn id(&self) -> &str { "analysis.complexity" }
    fn node_type(&self) -> &str { "ComplexityNode" }
    fn analyze(&self, input: &str) -> AnalysisResult {
        let score = compute_complexity(input);
        AnalysisResult {
            metrics: HashMap::from([(String::from("score"), score)]),
            explanation: String::from("Оценка цикломатической сложности"),
        }
    }
    fn explain(&self) -> String {
        "Оценка цикломатической сложности".into()
    }
}

pub fn register(registry: &mut NodeRegistry) {
    registry.add(Box::new(ComplexityNode));
}
```

Пример показывает добавление нового узла и регистрацию его в `NodeRegistry`.

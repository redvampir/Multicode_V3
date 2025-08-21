# Архитектура анализа

Документ описывает общий API узлов анализа, базовую иерархию типов и пример расширения системы на Rust.

## Модули высокого уровня

- **Базовый вычислительный узел** — основная обработка запросов и режим «без личности».
- **Модуль диалоговой логики** — отслеживание намерений пользователя и выбор стиля общения.
- **Модуль личности** — хранение устойчивого образа Нейры.
- **Модуль памяти и адаптации** — накопление опыта общения без разрушения базового ядра.
  Узлы памяти проходят [валидацию](memory-nodes.md#проверка-валидности-узлов)
  по метрикам, тестам и ручной проверке.
- **Модуль интересов, творчества и игр** — обучение через игры, генерация новых узлов анализа.
- **Модуль скепсиса и проверки** — вставка уточнений и проверка фактов.


## API узлов

Трейт `AnalysisNode` задаёт минимальный контракт для всех реализаций. Метод `analyze` возвращает структуру `AnalysisResult` с наборами метрик и текстовым объяснением, а `explain` выдаёт краткое описание логики узла. Регистрация конкретных реализаций производится через `NodeRegistry`.

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

## Иерархия узлов

```text
AnalysisNode
├─ DataSourceNode        # интеграция с внешними источниками данных
├─ ReasoningNode         # агрегирование и интерпретация результатов
└─ DomainNode            # логика для конкретных областей
   ├─ ProgrammingSyntaxNode
   ├─ NaturalLanguageNode
   └─ DomainSpecificNode
```

## Пример расширения на Rust

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
        "Оценивает цикломатическую сложность кода".into()
    }
}

pub fn register(registry: &mut NodeRegistry) {
    registry.add(Box::new(ComplexityNode));
}
```

Пример демонстрирует добавление нового узла и его регистрацию в `NodeRegistry`.

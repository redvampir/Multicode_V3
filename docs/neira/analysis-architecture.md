# Архитектура анализа

## Навигация
- [Обзор Нейры](README.md)
- [Узлы действий](action-nodes.md)
- [Узлы анализа](analysis-nodes.md)
- [Узлы памяти](memory-nodes.md)
- [Архитектура анализа](analysis-architecture.md)
- [Поддерживающие системы](support-systems.md)
- [Личность Нейры](personality.md)
- [Шаблон узла](node-template.md)
- [Политика источников](source-policy.md)

## Оглавление
- [Модули высокого уровня](#модули-высокого-уровня)
- [API узлов](#api-узлов)
- [Иерархия узлов](#иерархия-узлов)
- [Пример расширения на Rust](#пример-расширения-на-rust)


Документ описывает общий API узлов анализа, базовую иерархию типов и пример расширения системы на Rust.

## Модули высокого уровня

- **Базовый вычислительный узел** — основная обработка запросов и режим «без личности».
- **Модуль диалоговой логики** — отслеживание намерений пользователя и выбор стиля общения.
- **Модуль личности** — хранение устойчивого образа Нейры.
- **Модуль памяти и адаптации** — накопление опыта общения без разрушения базового ядра.
- **Модуль интересов, творчества и игр** — обучение через игры, генерация новых узлов анализа.
- **Модуль скепсиса и проверки** — вставка уточнений и проверка фактов.


## API узлов

Трейт `AnalysisNode` задаёт минимальный контракт для всех реализаций. Метод `analyze` возвращает структуру `AnalysisResult` с метриками качества и текстовым объяснением, а `explain` выдаёт краткое описание логики узла. Дополнительно интерфейс предоставляет текущий `status` и связи `links`. Регистрация конкретных реализаций производится через `NodeRegistry`.

```rust
pub trait AnalysisNode {
    fn id(&self) -> &str;
    fn analysis_type(&self) -> &str;
    fn status(&self) -> NodeStatus;
    fn links(&self) -> &[String];
    fn analyze(&self, input: &str) -> AnalysisResult;
    fn explain(&self) -> String;
}
```

Тип `AnalysisResult` содержит идентификатор, основной текстовый вывод, статус выполнения, метрики качества, цепочку рассуждений, ссылки и текстовое объяснение. Поля `id` и `output` обязательны и сериализуются строками. `quality_metrics` передаётся как структура `QualityMetrics { credibility, recency_days, demand }`, где `credibility` лежит в диапазоне `0..1`, `recency_days` измеряется в днях, а `demand` отражает количество запросов. Поле `metadata.schema` фиксирует версию схемы результата.

```rust
pub struct AnalysisResult {
    pub id: String,
    pub output: String,
    pub status: NodeStatus,
    pub quality_metrics: QualityMetrics,
    pub reasoning_chain: Vec<String>,
    pub explanation: String,
    pub links: Vec<String>,
    pub metadata: AnalysisMetadata,
}

pub struct QualityMetrics {
    pub credibility: f32,   // 0..1
    pub recency_days: u32,  // возраст данных
    pub demand: u32,        // число запросов
}

pub struct AnalysisMetadata {
    pub schema: String,
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

pub struct ComplexityNode;

impl AnalysisNode for ComplexityNode {
    fn id(&self) -> &str { "analysis.complexity" }
    fn analysis_type(&self) -> &str { "ComplexityNode" }
    fn status(&self) -> NodeStatus { NodeStatus::Active }
    fn links(&self) -> &[String] { &[] }
    fn analyze(&self, input: &str) -> AnalysisResult {
        let score = compute_complexity(input);
        AnalysisResult {
            id: self.id().into(),
            output: score.to_string(),
            status: NodeStatus::Active,
            quality_metrics: QualityMetrics {
                credibility: 0.0,
                recency_days: 0,
                demand: 0,
            },
            reasoning_chain: vec!["compute cyclomatic complexity".into()],
            explanation: String::from("Оценка цикломатической сложности"),
            links: vec![],
            metadata: AnalysisMetadata { schema: "1.0".into() },
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

## Схемы

JSON‑схемы расположены в каталоге [../../schemas](../../schemas). При несовместимых изменениях повышайте версию: `1.0.0` → `1.1.0`.

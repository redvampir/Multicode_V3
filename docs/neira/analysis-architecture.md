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

Трейт `AnalysisNode` задаёт минимальный контракт для всех реализаций. Метод `analyze` возвращает структуру `AnalysisResult` с метриками качества, оценкой неопределённости и цепочкой рассуждений, а `explain` выдаёт краткое описание логики узла. Дополнительно интерфейс предоставляет текущий `status`, связи `links` и порог `confidence_threshold`, при котором результат считается пригодным к использованию. Регистрация конкретных реализаций производится через `NodeRegistry`. Цепочка рассуждений (`reasoning_chain`) не хранится в самом узле и возвращается только в `AnalysisResult`.

```rust
pub trait AnalysisNode {
    fn id(&self) -> &str;
    fn analysis_type(&self) -> &str;
    fn status(&self) -> NodeStatus;
    fn links(&self) -> &[String];
    fn confidence_threshold(&self) -> f32;
    fn analyze(&self, input: &str) -> AnalysisResult;
    fn explain(&self) -> String;
}
```

Тип `AnalysisResult` содержит идентификатор, основной текстовый вывод, статус выполнения, метрики качества, цепочку рассуждений, показатель неопределённости, ссылки и текстовое объяснение. Поля `id` и `output` обязательны и сериализуются строками. `quality_metrics` передаётся как структура `QualityMetrics { credibility, recency_days, demand }`, где `credibility` лежит в диапазоне `0..1`, `recency_days` измеряется в днях, а `demand` отражает количество запросов. Поле `uncertainty_score` числом в диапазоне `0..1` характеризует риск ошибки (`0` — полная уверенность). Поле `metadata.schema` фиксирует версию схемы результата.

```rust
pub struct AnalysisResult {
    pub id: String,
    pub output: String,
    pub status: NodeStatus,
    pub quality_metrics: QualityMetrics,
    pub reasoning_chain: Vec<String>,
    pub uncertainty_score: f32,
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

Логические уровни образуют цепочку `Binary` → `Artistic`, а маршрутизация определяет переходы между ними в зависимости от входных данных.
На каждом уровне узлы возвращают `quality_metrics`, `reasoning_chain` и `uncertainty_score`.

## Пример расширения на Rust

```rust

pub struct ComplexityNode;

impl AnalysisNode for ComplexityNode {
    fn id(&self) -> &str { "analysis.complexity" }
    fn analysis_type(&self) -> &str { "ComplexityNode" }
    fn status(&self) -> NodeStatus { NodeStatus::Active }
    fn links(&self) -> &[String] { &[] }
    fn confidence_threshold(&self) -> f32 { 0.75 }
    fn analyze(&self, input: &str) -> AnalysisResult {
        let (cyclo, cognitive) = compute_complexity(input);
        let credibility = assess_sources(&[cyclo, cognitive]);
        let reasoning_chain = vec![
            format!("cyclomatic: {}", cyclo),
            format!("cognitive: {}", cognitive),
            "aggregate metrics".into(),
        ];
        AnalysisResult {
            id: self.id().into(),
            output: format!("{};{}", cyclo, cognitive),
            status: NodeStatus::Active,
            quality_metrics: QualityMetrics {
                credibility,
                recency_days: 0,
                demand: 0,
            },
            reasoning_chain,
            uncertainty_score: 1.0 - credibility,
            explanation: String::from("Комплексная оценка сложности"),
            links: vec![],
            metadata: AnalysisMetadata { schema: "1.0".into() },
        }
    }
    fn explain(&self) -> String {
        "Собирает несколько метрик сложности и проверяет источники".into()
    }
}

pub fn register(registry: &mut NodeRegistry) {
    registry.add(Box::new(ComplexityNode));
}
```

Пример демонстрирует добавление нового узла и его регистрацию в `NodeRegistry`.

## Схемы

JSON‑схемы расположены в каталоге [../../schemas](../../schemas):
- [node-template.schema.json](../../schemas/node-template.schema.json)
- [analysis-result.schema.json](../../schemas/analysis-result.schema.json)

При несовместимых изменениях повышайте версию: `1.0.0` → `1.1.0`.

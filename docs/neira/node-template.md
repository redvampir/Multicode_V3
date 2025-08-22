# NodeTemplate

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
- [Обязательные поля](#обязательные-поля)
- [Пример](#пример)
  - [JSON](#json)
  - [YAML](#yaml)
- [Проверка](#проверка)


Шаблон для создания узлов анализа. Все поля являются обязательными.

## Обязательные поля

| Поле | Описание |
| --- | --- |
| `id` | Уникальный идентификатор шаблона. |
| `analysis_type` | Тип создаваемого узла. |
| `links` | Список связей с другими узлами. |
| `draft_content` | Черновое содержимое узла. |
| `metadata` | Дополнительные метаданные в формате ключ‑значение. |

## Пример

### JSON

```json
{
  "id": "example.template",
  "analysis_type": "ProgrammingSyntaxNode",
  "links": ["prog.syntax.base"],
  "draft_content": "Initial description",
  "metadata": {
    "schema": "1.0",
    "source": "https://example.org"
  }
}
```

### YAML

```yaml
id: example.template
analysis_type: ProgrammingSyntaxNode
links:
  - prog.syntax.base
draft_content: Initial description
metadata:
  schema: "1.0"
  source: "https://example.org"
```

## Проверка

Файл можно проверить с помощью JSON Schema. Сохраните шаблон в файл и выполните:

```bash
npx ajv validate -s node-template.schema.json -d node-template.json
npx ajv validate -s node-template.schema.json -d node-template.yaml
```

## Схемы

JSON‑схемы расположены в каталоге [../../schemas](../../schemas). При несовместимых изменениях повышайте версию: `1.0.0` → `1.1.0`.

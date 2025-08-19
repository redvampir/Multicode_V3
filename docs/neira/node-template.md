# NodeTemplate

Шаблон для создания узлов анализа. Все поля являются обязательными.

## Обязательные поля

| Поле | Описание |
| --- | --- |
| `id` | Уникальный идентификатор шаблона. |
| `node_type` | Тип создаваемого узла. |
| `links` | Список связей с другими узлами. |
| `draft_content` | Черновое содержимое узла. |
| `metadata` | Дополнительные метаданные в формате ключ‑значение. |

## Пример

### JSON

```json
{
  "id": "example.template",
  "node_type": "ProgrammingSyntaxNode",
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
node_type: ProgrammingSyntaxNode
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

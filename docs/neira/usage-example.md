# Пример использования

Последовательность обработки запроса в Neira:

1. **Пользовательский запрос** — отправляется через CLI или API.
2. **InteractionHub** — принимает сообщение и определяет, какой узел активировать.
3. **AnalysisNode** — анализирует намерение и формирует план действий.
4. **MemoryNode** — извлекает или обновляет связанные записи.
5. **ActionNode** — выполняет команду (генерация кода, вывод данных и т.д.).
6. **Ответ** — результат возвращается пользователю вместе с трассировкой.

```bash
# запрос
curl -X POST http://localhost:4000/interact \
     -H 'Content-Type: application/json' \
     -d '{"message":"Список задач"}'

# ответ
{
  "reply": "Задачи: [\"task1\", \"task2\"]",
  "trace": [
    {"node": "AnalysisNode/main", "status": "ok"},
    {"node": "MemoryNode/tasks", "status": "hit"},
    {"node": "ActionNode/list", "result": ["task1", "task2"]}
  ]
}
```

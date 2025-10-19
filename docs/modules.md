# Модули

Ядро Multicode состоит из набора независимых модулей, которые подключаются через флаги Cargo.

| Флаг     | Возможность                           | Зависимости       |
| -------- | ------------------------------------- | ----------------- |
| `git`    | интеграция с системой контроля версий | `git2`            |
| `watch`  | отслеживание изменений файлов         | `notify`, `tokio` |
| `export` | экспорт визуальных представлений      | —                 |
| `db`     | хранение данных в базе                | `sqlx`, `tokio`   |

## Примеры сборки

```bash
# только интеграция с Git
cargo build -p core --no-default-features --features "git"

# только отслеживание изменений
cargo build -p core --no-default-features --features "watch"

# экспорт визуальных представлений
cargo build -p core --no-default-features --features "export"

# работа с базой данных
cargo build -p core --no-default-features --features "db"

# комбинирование модулей
cargo build -p core --no-default-features --features "git,watch"
```

Подробнее о плагинах см. в [plugin-guide.md](plugin-guide.md). Общий обзор возможностей приведён в [features.md](features.md). Структура каталогов описана в [repo-structure.md](repo-structure.md). Термины смотрите в [глоссарии](glossary.md).

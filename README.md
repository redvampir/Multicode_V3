# Multicode

Перед началом работы обязательно ознакомьтесь с [RULES.md](RULES.md). Инструкции для работы с искусственным ассистентом Codex находятся в [docs/assistant_manual.md](docs/assistant_manual.md).

## Обзор
Multicode V3 — редактор исходного кода, объединяющий текстовое и визуальное представления программ. Пользователь свободно переключается между режимами и может работать с дополнительными метаданными без изменения логики кода.

Проект поставляется как автономное десктоп‑приложение и не требует запуска web‑сервера.

## Архитектура
![Архитектура](docs/architecture.svg)

- **core (Rust):** базовая библиотека с опциональными модулями.
- **desktop (Rust, Iced):** десктоп‑приложение, использующее `core`.
- **legacy-backend:** архивный код предыдущей версии, подключается при необходимости.

## Структура рабочей области

- `core/` — библиотека ядра. Дополнительные возможности подключаются через флаги: `git`, `watch`, `export`, `db`.
- `desktop/` — оффлайн‑приложение на Iced.
- `plugins/` — плагины блоков.
- `examples/` — примеры использования.
- `legacy-backend/` — опциональный архивный backend.

## Windows prerequisites

Для сборки на Windows через окружение MSYS2 установите следующие пакеты:

- `gtk3`
- `pkg-config`
- `glib2`
- `cairo`
- `pango`

Используйте следующую команду в терминале MSYS2:

```bash
pacman -S --noconfirm mingw-w64-x86_64-gtk3 pkg-config
```

Убедитесь, что путь `MSYS2\mingw64\bin` добавлен в переменную окружения `PATH`.

## Быстрый старт

```bash
cargo run -p desktop                  # запуск приложения
cargo test -p core                    # тесты ядра
cargo build --release -p desktop      # релизная сборка
```

Для выборочной сборки ядра с дополнительными возможностями:

```bash
cargo build -p core --no-default-features --features "git,watch"
```

## Стандартные проверки

Перед отправкой изменений убедитесь, что основные проверки проходят без ошибок:

```bash
cargo fmt -- --check
cargo clippy -- -D warnings
cargo test
cargo audit
```

## Лицензия
Проект распространяется по лицензии MIT. Полный текст приведён в файле [LICENSE](LICENSE).

## Ручное тестирование

Сценарии ручной проверки описаны в [NODE_EDITOR_MANUAL_TESTS.md](NODE_EDITOR_MANUAL_TESTS.md).

```bash
cargo run -p desktop -- examples/test.js
```


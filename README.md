# Multicode

Перед началом работы обязательно ознакомьтесь с [RULES.md](RULES.md).


## Обзор
Multicode V3 — редактор исходного кода, который объединяет текстовое и визуальное представления программ. Пользователь может свободно переключаться между режимами и работать с дополнительными метаданными без изменения логики кода.

## Архитектура
- **Backend (Rust, Tauri):** парсинг файлов через tree-sitter, хранение и обработка метаданных, управление плагинами.
- **Frontend (Vite, JavaScript):** интерфейс с Monaco‑редактором и canvas‑основанным редактором блоков.
- **Связь компонентов:** локальный WebSocket‑сервер синхронизирует состояния между частями приложения.

## Ключевые возможности
- Переключение между текстовым и визуальным режимами.
- Сохранение метаданных в комментариях к исходному коду.
- Поддержка плагинов для расширения набора блоков.
- Работа с языками Rust, Python, JavaScript, TypeScript, CSS, HTML и Go (см. [backend/src/parser/mod.rs](backend/src/parser/mod.rs)).

## Структура метакомментариев
Служебная информация сохраняется в комментариях `@VISUAL_META` в формате JSON. Актуальное определение описано в [backend/meta.schema.json](backend/meta.schema.json).

Доступные поля:
- `id` (`string`, обязательное) — идентификатор, связывающий метаданные с блоком.
- `version` (`integer`, по умолчанию 1) — версия схемы метаданных.
- `x` (`number`, обязательное) — координата X на холсте.
- `y` (`number`, обязательное) — координата Y на холсте.
- `tags` (`string[]`) — произвольные теги, связанные с блоком.
- `links` (`string[]`) — ссылки на другие блоки по `id`.
- `extends` (`string`) — идентификатор записи, из которой наследуются данные.
- `origin` (`string`) — обратный путь до исходного внешнего файла.
- `translations` (`object`) — переводы подписей блока по коду языка.
- `ai` (`object`) — заметка, сгенерированная ИИ:
  - `description` (`string`) — описание от ИИ.
  - `hints` (`string[]`) — полезные подсказки.
- `extras` (`object`) — произвольные данные плагинов.
- `updated_at` (`string`, формат `date-time`, обязательное) — время последнего обновления в UTC.

Пример:
```json
{
  "id": "child",
  "version": 1,
  "x": 42,
  "y": 13,
  "tags": ["math", "demo"],
  "links": ["parent"],
  "extends": "parent",
  "origin": "path/to/original",
  "translations": { "en": "Child block", "ru": "Дочерний блок" },
  "ai": {
    "description": "Automatically summarized",
    "hints": ["Consider refactoring"]
  },
  "extras": { "plugin": "value" },
  "updated_at": "2024-01-01T00:00:00Z"
}
```
Комментарий не влияет на выполнение программы и может быть удалён при экспорте.

## Установка окружения
1. **Node.js**
   - Скачайте пакет с [nodejs.org](https://nodejs.org) или установите через менеджер `nvm`:
     ```bash
     nvm install --lts
     ```
2. **Rust**
   - Установите через `rustup`:
     ```bash
     curl https://sh.rustup.rs -sSf | sh
     ```
   - После установки перезапустите терминал и выполните `rustc --version`.
3. **Tauri CLI и зависимости**
   - Проект и его зависимости используют **Tauri v2**. Установите CLI версии 2:
     ```bash
     npm install -g @tauri-apps/cli@^2
     ```
   - Либо через Cargo:
     ```bash
     cargo install tauri-cli --version ^2
     ```
   - Проверьте установку командой `tauri --version`.
4. **Системные библиотеки GTK/GLib**
   - Установите пакеты:
     ```bash
     sudo apt-get install -y libglib2.0-dev libgtk-3-dev pkg-config
     ```
   - После установки задайте переменную окружения `PKG_CONFIG_PATH`, указывающую на каталог с файлами `gobject-2.0.pc` и `glib-2.0.pc` (обычно `/usr/lib/x86_64-linux-gnu/pkgconfig` или `/usr/lib/pkgconfig`).
   - Добавьте строку `export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig` в `~/.bashrc` или аналогичный профиль, чтобы переменная сохранялась между сессиями.

## Инструкции по запуску и сборке
1. **Клонирование репозитория**
   ```bash
   git clone https://github.com/<user>/Multicode_V3.git
   cd Multicode_V3
   ```
2. **Проверка версий Node.js и Rust**
   ```bash
   node -v
   rustc --version
   ```
3. **Установка зависимостей**
   - Frontend
     ```bash
     cd frontend
     npm install
     ```
   - Backend
     ```bash
     cd ../backend
     cargo build
     ```
4. **Запуск в режиме разработки**
   - Backend отдельно
     ```bash
     cd backend
     cargo run
     ```
   - Frontend вместе с бэкендом
     ```bash
     cd frontend
     npm run tauri dev
     ```
5. **Сборка релизной версии**
   - Backend
     ```bash
     cd backend
     cargo build --release
     ```
   - Frontend
     ```bash
     cd frontend
     npm run tauri build
     ```
     Скомпилированные бинарные файлы появятся в `frontend/src-tauri/target/release`.
6. **Запуск тестов**
   - Frontend
     ```bash
     cd frontend
     npm test
     ```
   - Backend
     ```bash
     cd backend
     cargo test
     ```

### Устранение неполадок
- Ошибки `GTK`/`GLib` или `pkg-config`: установите пакеты `libglib2.0-dev`, `libgtk-3-dev`, `pkg-config` и проверьте `PKG_CONFIG_PATH`.
- Тесты `npm test` или `cargo test` падают: переустановите зависимости (`npm ci`, `cargo clean && cargo test`) и убедитесь в корректности версий Node.js и Rust.

## Использование CLI
Бэкенд содержит минимальный интерфейс командной строки для работы с кодом и метаданными. Примеры запуска:

```
cargo run -- parse path/to/file.rs --lang rust
cargo run -- export input.rs output.rs --strip-meta
cargo run -- meta list path/to/file.rs
cargo run -- meta fix path/to/file.rs
cargo run -- meta remove path/to/file.rs
```

## Добавление модулей
Плагины позволяют добавлять новые блоки и функциональность. Минимальный
пример расположен в [examples/plugins](examples/plugins) и имеет следующую
структуру:

```
examples/plugins/
├── my_plugin.rs     — backend-часть
├── my-block.js      — frontend-часть
└── README.md
```

### Пошаговое создание плагина
1. Скопируйте структуру каталога из примера или создайте аналогичный
   набор файлов.
2. **Backend:** реализуйте трейт [`Plugin`](backend/src/plugins/mod.rs),
   возвращающий `BlockDescriptor` с описанием нового блока.
3. Подключите плагин на серверной стороне и соберите проект.
4. **Frontend:** реализуйте модуль с функцией `register({ Block,
   registerBlock })`, в которой объявите класс блока и вызовете
   `registerBlock`.
5. Передайте путь к модулю в `loadBlockPlugins`, например
   `loadBlockPlugins(['./my-block.js'])`.

### Тестирование плагинов
- Запускайте `cargo test` в каталоге backend для проверки серверной части.
- Выполняйте `npm test` в каталоге frontend, чтобы убедиться в корректной
  работе визуальной компоненты.
- Для интеграционной проверки запускайте приложение командой
  `npm run dev` и убедитесь, что новый блок отображается без ошибок.

## Словарь
- **Блок:** визуальное представление участка кода.
- **Метакомментарий:** комментарий `@VISUAL_META` с координатами и прочими данными.
- **Плагин:** набор расширений для добавления новых блоков.

## Структура репозитория
- `backend` — серверная часть на Rust.
- `frontend` — интерфейс и клиентская логика.
- `plugins` — встроенные плагины и механизм их загрузки.
- `examples` — примеры плагинов и использования.
- `logs` — временные файлы и журналы backend (файлы вида `backend.log.YYYY-MM-DD`).

## Лицензия
Проект распространяется по лицензии MIT. Полный текст приведён в файле [LICENSE](LICENSE).

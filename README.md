# Multicode

Перед началом работы обязательно ознакомьтесь с [RULES.md](RULES.md).


## Обзор
Multicode V3 — редактор исходного кода, который объединяет текстовое и визуальное представления программ. Пользователь может свободно переключаться между режимами и работать с дополнительными метаданными без изменения логики кода.

## Архитектура
![Архитектура](docs/architecture.svg)

- **Backend (Rust, Tauri):** парсинг файлов через tree-sitter, хранение и обработка метаданных, управление плагинами.
- **Frontend (Vite, JavaScript):** интерфейс с Monaco‑редактором и canvas‑основанным редактором блоков.
- **Локальный WebSocket‑сервер:** синхронизирует состояния между частями приложения.

## Ключевые возможности
- Переключение между текстовым и визуальным режимами.
- Сохранение метаданных в комментариях к исходному коду.
- Поддержка плагинов для расширения набора блоков.
- Работа с несколькими языками программирования:
  - Rust
  - Python
  - JavaScript
  - TypeScript
  - CSS
  - HTML
  - Go
  *(см. [backend/src/parser/mod.rs](backend/src/parser/mod.rs))*

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

Примеры использования метакомментариев в исходном коде:

```rust
// @VISUAL_META {"id": "node", "x": 0, "y": 0}
```

```javascript
// @VISUAL_META {"id": "node", "x": 0, "y": 0}
```

```typescript
// @VISUAL_META {"id": "node", "x": 0, "y": 0}
```

```go
// @VISUAL_META {"id": "node", "x": 0, "y": 0}
```

```python
# @VISUAL_META {"id": "node", "x": 0, "y": 0}
```

```css
/* @VISUAL_META {"id": "node", "x": 0, "y": 0} */
```

```html
<!-- @VISUAL_META {"id": "node", "x": 0, "y": 0} -->
```

Комментарий не влияет на выполнение программы и может быть удалён при экспорте.

## Установка окружения
Команды ниже приведены для macOS/Linux (zsh/bash) и Windows (PowerShell).

1. **Node.js**
   - macOS/Linux:
     ```bash
     nvm install --lts
     ```
   - Windows:
     ```powershell
     winget install OpenJS.NodeJS.LTS
     # или choco install nodejs-lts
     ```
2. **Rust**
   - macOS/Linux:
     ```bash
     curl https://sh.rustup.rs -sSf | sh
     ```
   - Windows:
     ```powershell
     winget install Rustlang.Rustup
     # или choco install rustup
     ```
   - Для сборки необходим установленный MSVC (Visual Studio Build Tools).
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
   - Подробности см. в официальной документации Tauri для [Windows](https://v2.tauri.app/start/prerequisites/#windows) и [macOS](https://v2.tauri.app/start/prerequisites/#macos).
4. **Системные библиотеки GTK/GLib**
   - Linux:
     ```bash
     sudo apt-get install -y libglib2.0-dev libgtk-3-dev pkg-config
     export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig
     ```
   - Windows:
     - Установите GTK/GLib (например, через [MSYS2](https://www.msys2.org/)).
     - Укажите путь к их `pkgconfig`:
       ```powershell
       $env:PKG_CONFIG_PATH="C:\\msys64\\mingw64\\lib\\pkgconfig"
       ```
   - macOS:
     ```bash
     brew install gtk+3 glib
     export PKG_CONFIG_PATH="/opt/homebrew/lib/pkgconfig"
     ```

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
Бэкенд содержит интерфейс командной строки для работы с кодом и метаданными. Подробную помощь по всем командам можно получить командой:

```
cargo run -- --help
```

### Команды

#### `parse`
Разбирает исходный файл и выводит информацию о блоках в формате JSON.

Параметры:
- `path` — путь к исходному файлу.
- `--lang <LANG>` — язык исходного файла.

#### `export`
Экспортирует файл, при необходимости удаляя метакомментарии `@VISUAL_META`.

Параметры:
- `input` — путь к исходному файлу.
- `output` — путь для сохранённого результата.
- `--strip-meta` — флаг удаления метакомментариев.

#### `meta list`
Показывает все метакомментарии из файла в формате JSON.

Параметры:
- `path` — путь к исходному файлу.

#### `meta fix`
Исправляет проблемы в метаданных, например дубликаты идентификаторов.

Параметры:
- `path` — путь к исходному файлу.

#### `meta remove`
Удаляет все метакомментарии из файла.

Параметры:
- `path` — путь к исходному файлу.

### Краткие примеры

| Команда | Описание | Пример |
| --- | --- | --- |
| `parse` | Парсит файл и выводит блоки | `cargo run -- parse path/to/file.rs --lang rust` |
| `export` | Экспортирует файл (при `--strip-meta` удаляет мету) | `cargo run -- export input.rs output.rs --strip-meta` |
| `meta list` | Показывает метакомментарии | `cargo run -- meta list path/to/file.rs` |
| `meta fix` | Исправляет метаданные | `cargo run -- meta fix path/to/file.rs` |
| `meta remove` | Удаляет метакомментарии | `cargo run -- meta remove path/to/file.rs` |

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
2. **Backend:** реализуйте трейт [`Plugin`](backend/src/plugins/mod.rs).
   Интерфейс определяет три метода: `name()` — уникальное имя плагина,
   `version()` — целевая версия API и `blocks()` — список
   `BlockDescriptor` с описанием новых блоков.
3. Скомпилируйте серверную часть в WebAssembly и подключите её к
   приложению:
   ```bash
   rustup target add wasm32-unknown-unknown
   cargo build --target wasm32-unknown-unknown --release
   cp target/wasm32-unknown-unknown/release/<имя>.wasm ../../plugins/
   ```
   Backend загружает `.wasm` через `WasmPlugin` и оборачивает его в
   реализацию трейта `Plugin`.
4. **Frontend:** реализуйте модуль с функцией
   `register({ Block, registerBlock })`, объявите класс блока и вызовите
   `registerBlock`. Пример:
   ```javascript
   export function register({ Block, registerBlock }) {
     class MyBlock extends Block {}
     registerBlock('MyBlock', MyBlock);
   }
   ```
5. Передайте путь к модулю в `loadBlockPlugins`, например
   `loadBlockPlugins(['./my-block.js'])`. Клиент запрашивает список
   модулей у сервера и регистрирует блоки, получая метаданные через
   WebSocket.

#### Сборка примера
Рабочий пример находится в каталоге
[examples/plugins](examples/plugins). После создания `Cargo.toml` для
плагина выполните команды:
```bash
cd examples/plugins
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```
Полученный `.wasm` скопируйте в каталог `plugins/`, а файл `my-block.js`
подключите через `loadBlockPlugins`.

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

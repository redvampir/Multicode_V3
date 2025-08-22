# Правила разработки для Multicode V3

## 🚀 О проекте

**Multicode V3** - это инновационный редактор исходного кода, который революционизирует процесс программирования, объединяя текстовое и визуальное представления программ. Мы создаём инструмент будущего, где код можно как писать, так и "рисовать". Rust обеспечивает производительность и надёжность backend'а, а JavaScript с Vite создаёт отзывчивый пользовательский интерфейс.

## 🛠 Начало работы

```bash
git clone https://github.com/redvampir/Multicode_V3.git
cd Multicode_V3

# Установка зависимостей frontend
cd frontend && npm install && npm test

# Проверка backend
cd ../backend && cargo test

# Возврат в корень для запуска
cd ..
npm run dev
```

### Системные требования
- **Node.js** (LTS версия)
- **Rust** (последняя стабильная версия)
- **Tauri v2** (CLI и зависимости, критически важно!)
- **GTK/GLib** библиотеки для Linux

## Как принять участие

1. Сделайте форк репозитория и клонируйте его локально.
2. Создайте новую ветку для своей задачи.
3. Установите зависимости для frontend и backend.
4. Перед отправкой PR запустите проверки:
   - `cargo fmt -- --check`, `cargo clippy` и `cargo audit` для Rust;
   - `npm run lint`, `npm test` и `npm audit` для JavaScript/TypeScript;
5. Убедитесь, что все команды завершились без ошибок.
6. Отправьте pull request с описанием изменений.

Подробные рекомендации по уровням тестирования и бенчмаркам для модуля Neira см. [docs/neira/testing.md](docs/neira/testing.md).


### Обновление lock-файлов

При выпуске новых патчей необходимо обновлять файлы зависимостей:

- `cargo update` актуализирует `Cargo.lock`;
- `npm update --package-lock-only` обновляет `package-lock.json`.

Убедитесь, что обновлённые lock-файлы входят в релизный коммит.

### Настройка форматеров

- **Rust**: убедитесь, что установлены компоненты `rustfmt` и `clippy`:
  ```bash
  rustup component add rustfmt clippy
  ```
- **JavaScript/TypeScript**: установите и настройте `eslint` и `prettier`:
  ```bash
  cd frontend
  npm install --save-dev eslint prettier
  ```
  Создайте `.eslintrc` и `.prettierrc` по необходимости.

### Стиль коммитов и оформление PR

- Мы рекомендуем использовать [Conventional Commits](https://www.conventionalcommits.org/ru/v1.0.0/).
- В описании PR кратко укажите, что было изменено, и добавьте ссылки на связанные issue.

## Проверка схем

JSON‑схемы хранятся в каталоге `schemas/`. В конвейере CI они валидируются командой:

```bash
npx ajv validate -s schemas/node-template.schema.json -d node-template.json
npx ajv validate -s schemas/analysis-node.schema.json -d analysis-node.json
```

Запускайте эту проверку локально при изменении схем, чтобы убедиться в корректности.

## 🏗 Архитектурные принципы

При разработке следуйте ключевым принципам Multicode:
- **Бесшовность** - переходы между режимами должны быть мгновенными
- **Неинвазивность** - метаданные не влияют на логику кода
- **Расширяемость** - система плагинов должна быть интуитивной
- **Производительность** - отзывчивость интерфейса превыше всего
- **Совместимость** - поддержка множества языков программирования

## 📁 Структура проекта

```
multicode/
├── backend/                    # Rust backend (Tauri)
│   ├── src/
│   │   ├── parsing/           # Tree-sitter парсеры
│   │   ├── metadata/          # Обработка метакомментариев
│   │   ├── plugins/           # Система плагинов
│   │   ├── websocket/         # WebSocket сервер
│   │   └── cli/               # Интерфейс командной строки
│   └── tests/                 # Тесты backend
├── frontend/                   # Vite + JavaScript frontend
│   ├── src/
│   │   ├── editors/           # Monaco и блочный редакторы
│   │   ├── canvas/            # Canvas-основанный движок
│   │   ├── plugins/           # Frontend плагины
│   │   ├── websocket/         # WebSocket клиент
│   │   └── components/        # UI компоненты
│   └── tests/                 # Frontend тесты
├── plugins/                    # Встроенные плагины
├── examples/                   # Примеры и документация
└── logs/                      # Логи разработки
```

## 🎯 Система метакомментариев

### Формат метаданных
Все визуальные метаданные сохраняются в специальных комментариях:

```javascript
// @VISUAL_META {"id": "block_123", "x": 100, "y": 200, "note": "Основная логика"}
function mainLogic() {
    // ваш код здесь
}
```

### Правила работы с метаданными:
1. Метакомментарии должны быть **неинвазивными**
2. Удаление метаданных не ломает функциональность
3. Формат JSON должен быть валидным
4. ID блоков должны быть уникальными в рамках файла

## 🎨 Кодстайл и конвенции

### Backend (Rust)
```rust
// Используйте описательные имена для публичных структур
pub struct CodeBlockMetadata {
    pub id: String,
    pub position: Position,
    pub visual_properties: VisualProps,
}

// Функции парсинга должны быть безопасными
pub fn parse_source_file(content: &str, language: Language) -> Result<ParsedCode, ParseError> {
    // Всегда обрабатывайте ошибки явно
    let tree = match parser.parse(content, None) {
        Some(tree) => tree,
        None => return Err(ParseError::InvalidSyntax),
    };
    
    Ok(ParsedCode::new(tree))
}

// Тесты должны покрывать граничные случаи
#[cfg(test)]
mod tests {
    #[test]
    fn test_metadata_parsing_with_malformed_json() {
        // Тестируем обработку некорректных метаданных
    }
}
```

### Frontend (JavaScript)
```javascript
// Используйте современный JavaScript
class VisualBlock {
    constructor(metadata, codeNode) {
        this.id = metadata.id;
        this.position = { x: metadata.x, y: metadata.y };
        this.codeNode = codeNode;
        this.connections = new Map();
    }
    
    // Методы должны быть chainable где возможно
    setPosition(x, y) {
        this.position = { x, y };
        this.updateVisualRepresentation();
        return this;
    }
    
    // Обработка ошибок через try-catch
    async syncWithBackend() {
        try {
            await this.websocket.send({
                type: 'UPDATE_BLOCK',
                data: this.serialize()
            });
        } catch (error) {
            console.error('Failed to sync block:', error);
            this.handleSyncError(error);
        }
    }
}

// Константы выносим в отдельные файлы
export const BLOCK_TYPES = {
    FUNCTION: 'function',
    CLASS: 'class',
    CONDITIONAL: 'conditional',
    LOOP: 'loop'
};
```

## 🔌 Разработка плагинов

### Структура плагина
```
my_plugin/
├── backend.rs              # Rust-часть плагина
├── frontend.js             # JavaScript-часть
├── metadata.json           # Описание плагина
└── README.md              # Документация
```

### Backend плагин (Rust)
```rust
use crate::plugins::{Plugin, BlockDescriptor, PluginResult};

pub struct MyCustomPlugin;

impl Plugin for MyCustomPlugin {
    fn name(&self) -> &str {
        "My Custom Plugin"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn get_block_descriptors(&self) -> Vec<BlockDescriptor> {
        vec![
            BlockDescriptor {
                id: "custom_block".to_string(),
                name: "Custom Block".to_string(),
                category: "Custom".to_string(),
                syntax_patterns: vec!["custom_syntax".to_string()],
            }
        ]
    }
    
    fn initialize(&mut self) -> PluginResult<()> {
        // Инициализация плагина
        println!("Custom plugin initialized");
        Ok(())
    }
}
```

### Frontend плагин (JavaScript)
```javascript
export function register({ Block, registerBlock }) {
    class CustomBlock extends Block {
        constructor(metadata) {
            super(metadata);
            this.type = 'custom_block';
        }
        
        render(ctx) {
            // Отрисовка блока на canvas
            ctx.fillStyle = '#ff6b6b';
            ctx.fillRect(this.x, this.y, this.width, this.height);
            
            // Добавляем текст
            ctx.fillStyle = '#ffffff';
            ctx.font = '14px Arial';
            ctx.fillText('Custom Block', this.x + 10, this.y + 20);
        }
        
        onDoubleClick() {
            // Обработка двойного клика
            this.openPropertiesDialog();
        }
    }
    
    // Регистрируем блок в системе
    registerBlock('custom_block', CustomBlock);
}
```

## 🧪 Тестирование

### Обязательные тесты для backend
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_valid_metadata() {
        let code = r#"
            // @VISUAL_META {"id": "test", "x": 10, "y": 20}
            fn test_function() {}
        "#;
        
        let result = parse_metadata(code);
        assert!(result.is_ok());
        
        let metadata = result.unwrap();
        assert_eq!(metadata.len(), 1);
        assert_eq!(metadata[0].id, "test");
    }
    
    #[test]
    fn test_websocket_synchronization() {
        // Тест синхронизации через WebSocket
    }
    
    #[test]
    fn test_plugin_loading() {
        // Тест загрузки плагинов
    }
}
```

### Frontend тесты
```javascript
import { describe, it, expect } from 'vitest';
import { VisualBlock } from '../src/canvas/VisualBlock.js';

describe('VisualBlock', () => {
    it('should create block with correct metadata', () => {
        const metadata = { id: 'test', x: 100, y: 200 };
        const block = new VisualBlock(metadata);
        
        expect(block.id).toBe('test');
        expect(block.position).toEqual({ x: 100, y: 200 });
    });
    
    it('should update position correctly', () => {
        const block = new VisualBlock({ id: 'test', x: 0, y: 0 });
        block.setPosition(50, 75);
        
        expect(block.position).toEqual({ x: 50, y: 75 });
    });
});
```

## 🚀 Git workflow

### Именование веток
- `main` — стабильная версия
- `develop` — активная разработка
- `feature/visual-debugging` — новые возможности
- `fix/websocket-connection` — исправления багов
- `plugin/rust-analyzer` — новые плагины
- `performance/canvas-optimization` — оптимизации

### Коммиты
Используйте conventional commits:
```bash
feat(frontend): добавлен drag-and-drop для блоков
fix(backend): исправлена синхронизация метаданных
perf(canvas): оптимизирована отрисовка больших файлов
plugin: добавлен плагин для Python
docs: обновлена документация по созданию плагинов
```

### Pre-commit проверки
```bash
# Проверяем все тесты
npm run test
cargo test

# Форматирование кода
cargo fmt --all
npm run format

# Линтинг
cargo clippy -- -D warnings
npm run lint
```

## ⚡ Производительность

### Backend оптимизации
- Используйте `Arc<T>` для разделяемых данных
- Применяйте асинхронность для I/O операций
- Кэшируйте результаты парсинга
- Минимизируйте клонирование больших структур

### Frontend оптимизации
- Используйте `requestAnimationFrame` для анимаций
- Применяйте виртуализацию для больших списков блоков
- Дебаунсите частые операции (изменение размера, скролл)
- Кэшируйте результаты рендеринга

## 🔧 CLI и автоматизация

```bash
# Парсинг файла
cargo run -- parse src/main.rs --lang rust

# Экспорт без метаданных
cargo run -- export input.js output.js --strip-meta

# Управление метаданными
cargo run -- meta list src/
cargo run -- meta fix src/main.rs
cargo run -- meta remove src/components/
```

## 📚 Документация

### Требования к документации
- Каждый публичный API должен иметь rustdoc/jsdoc
- Примеры использования для сложных функций
- Диаграммы архитектуры для новых модулей
- Changelog для breaking changes

### Пример документации
```rust
/// Парсит исходный код и извлекает метаданные визуальных блоков.
/// 
/// # Arguments
/// 
/// * `content` - Содержимое исходного файла
/// * `language` - Язык программирования для парсинга
/// 
/// # Returns
/// 
/// Возвращает `Ok(Vec<BlockMetadata>)` при успешном парсинге,
/// или `Err(ParseError)` при ошибке.
/// 
/// # Example
/// 
/// ```rust
/// let code = "// @VISUAL_META {...}\nfn main() {}";
/// let metadata = parse_visual_metadata(code, Language::Rust)?;
/// assert_eq!(metadata.len(), 1);
/// ```
pub fn parse_visual_metadata(content: &str, language: Language) -> Result<Vec<BlockMetadata>, ParseError>
```

## ✅ Checklist перед релизом

### Backend
- [ ] Все тесты проходят (`cargo test`)
- [ ] Clippy не выдаёт предупреждений
- [ ] Форматирование соответствует стандарту (`cargo fmt`)
- [ ] WebSocket сервер стабильно работает
- [ ] CLI команды функционируют корректно
- [ ] Плагины загружаются без ошибок

### Frontend
- [ ] Все тесты проходят (`npm test`)
- [ ] Линтер не выдаёт ошибок (`npm run lint`)
- [ ] Monaco редактор синхронизируется с визуальным
- [ ] Canvas рендеринг работает плавно
- [ ] Переключение режимов происходит мгновенно
- [ ] Плагины регистрируются корректно

### Интеграция
- [ ] WebSocket соединение стабильно
- [ ] Метаданные сохраняются и загружаются
- [ ] Поддерживаются все заявленные языки
- [ ] Экспорт/импорт работает корректно
- [ ] Документация актуальна

## 🎨 Дизайн-система

### Принципы UI/UX
- **Минимализм** - чистый интерфейс без лишних элементов
- **Отзывчивость** - мгновенный feedback на действия пользователя
- **Консистентность** - единый стиль во всех компонентах
- **Доступность** - поддержка клавиатурной навигации
- **Темизация** - поддержка светлой и тёмной тем

### Цветовая палитра
```css
:root {
  --primary: #007acc;
  --secondary: #6c757d;
  --success: #28a745;
  --warning: #ffc107;
  --error: #dc3545;
  --background: #1e1e1e;
  --surface: #252526;
  --text: #cccccc;
}
```

## 🤖 Правила для ИИ-помощников (Codex)

### Ограничения размера файлов
При генерации кода соблюдайте следующие лимиты:

- **Rust файлы:** максимум 300 строк на файл
- **JavaScript файлы:** максимум 250 строк на файл  
- **Конфигурационные файлы:** максимум 100 строк
- **Тестовые файлы:** максимум 200 строк на тест-модуль

Если функциональность превышает лимит, разбивайте на модули:
```rust
// ❌ Плохо - монолитный файл на 500+ строк
mod everything_in_one_file;

// ✅ Хорошо - разделение по ответственности
mod parser;
mod metadata;
mod websocket;
```

### Требования к комментариям
**ВСЕ комментарии ОБЯЗАТЕЛЬНО на русском языке:**

```rust
// ✅ Правильно
/// Парсит метаданные из комментариев в исходном коде
/// 
/// # Аргументы
/// * `content` - Содержимое файла для анализа
/// * `language` - Язык программирования
pub fn parse_metadata(content: &str, language: Language) -> Result<Vec<Metadata>, Error> {
    // Ищем паттерн @VISUAL_META в комментариях
    let pattern = Regex::new(r"@VISUAL_META\s*(\{.*?\})")?;
    
    // Обрабатываем каждое совпадение
    for capture in pattern.captures_iter(content) {
        // Парсим JSON из комментария
        let metadata_json = &capture[1];
        // ... остальная логика
    }
}

// ❌ Неправильно - английские комментарии
/// Parses metadata from source code comments
pub fn parse_metadata(content: &str) -> Result<Vec<Metadata>, Error> {
    // Find @VISUAL_META pattern
    // ...
}
```

### Частые ошибки ИИ - ИЗБЕГАТЬ!

#### 1. Забывчивость про обработку ошибок
```rust
// ❌ ИИ часто забывает про ошибки
fn read_file(path: &str) -> String {
    std::fs::read_to_string(path).unwrap() // НИКОГДА так не делать!
}

// ✅ Правильная обработка ошибок
fn read_file(path: &str) -> Result<String, io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(e) => {
            log::error!("Не удалось прочитать файл {}: {}", path, e);
            Err(e)
        }
    }
}
```

#### 2. Игнорирование граничных случаев
```rust
// ❌ ИИ забывает про пустые входные данные
fn process_blocks(blocks: Vec<Block>) -> Vec<ProcessedBlock> {
    blocks.iter().map(|b| process_single_block(b)).collect()
}

// ✅ Учитываем граничные случаи
fn process_blocks(blocks: Vec<Block>) -> Result<Vec<ProcessedBlock>, ProcessError> {
    if blocks.is_empty() {
        log::warn!("Получен пустой список блоков для обработки");
        return Ok(Vec::new());
    }
    
    let mut results = Vec::with_capacity(blocks.len());
    for block in blocks {
        match process_single_block(&block) {
            Ok(processed) => results.push(processed),
            Err(e) => {
                log::error!("Ошибка обработки блока {}: {}", block.id, e);
                return Err(e);
            }
        }
    }
    Ok(results)
}
```

#### 3. Неправильная работа с асинхронностью
```javascript
// ❌ ИИ забывает про async/await
function loadData() {
    fetch('/api/data')
        .then(response => response.json())
        .then(data => {
            // Обработка данных без учета ошибок
            updateUI(data);
        });
}

// ✅ Правильная асинхронная обработка
async function loadData() {
    try {
        const response = await fetch('/api/data');
        
        if (!response.ok) {
            throw new Error(`HTTP ошибка! статус: ${response.status}`);
        }
        
        const data = await response.json();
        updateUI(data);
        
    } catch (error) {
        console.error('Ошибка загрузки данных:', error);
        showErrorMessage('Не удалось загрузить данные. Попробуйте позже.');
    }
}
```

#### 4. Отсутствие валидации входных данных
```javascript
// ❌ ИИ часто пропускает валидацию
function createBlock(metadata) {
    return new Block(metadata.id, metadata.x, metadata.y);
}

// ✅ Тщательная валидация
function createBlock(metadata) {
    // Проверяем наличие обязательных полей
    if (!metadata || typeof metadata !== 'object') {
        throw new Error('Метаданные блока должны быть объектом');
    }
    
    if (!metadata.id || typeof metadata.id !== 'string') {
        throw new Error('ID блока обязателен и должен быть строкой');
    }
    
    const x = Number(metadata.x);
    const y = Number(metadata.y);
    
    if (isNaN(x) || isNaN(y)) {
        throw new Error('Координаты блока должны быть числами');
    }
    
    if (x < 0 || y < 0) {
        throw new Error('Координаты блока не могут быть отрицательными');
    }
    
    return new Block(metadata.id, x, y);
}
```

#### 5. Неэффективная работа с памятью
```rust
// ❌ ИИ создает лишние клоны
fn process_files(files: &[String]) -> Vec<String> {
    files.iter()
        .map(|file| file.clone()) // Ненужное клонирование!
        .filter(|f| f.ends_with(".rs"))
        .collect()
}

// ✅ Эффективная работа с референсами
fn process_files(files: &[String]) -> Vec<&String> {
    files.iter()
        .filter(|file| file.ends_with(".rs"))
        .collect()
}
```

#### 6. Игнорирование логирования
```rust
// ❌ ИИ забывает про логи
fn connect_websocket(url: &str) -> Result<WebSocket, Error> {
    WebSocket::connect(url)
}

// ✅ Подробное логирование
fn connect_websocket(url: &str) -> Result<WebSocket, Error> {
    log::info!("Попытка подключения к WebSocket: {}", url);
    
    match WebSocket::connect(url) {
        Ok(ws) => {
            log::info!("Успешно подключились к WebSocket");
            Ok(ws)
        },
        Err(e) => {
            log::error!("Ошибка подключения к WebSocket {}: {}", url, e);
            Err(e)
        }
    }
}
```

### Чек-лист для ИИ перед генерацией кода

- [ ] **Размер файла** не превышает лимиты
- [ ] **Все комментарии на русском языке**
- [ ] **Обработка ошибок** во всех критических местах
- [ ] **Валидация входных данных** добавлена
- [ ] **Граничные случаи** учтены (пустые массивы, null, undefined)
- [ ] **Логирование** добавлено для отладки
- [ ] **Асинхронность** обработана корректно
- [ ] **Память используется эффективно** (минимум клонирований)
- [ ] **Тесты покрывают** основные сценарии
- [ ] **Документация** соответствует коду

### Примеры качественного кода для ИИ

```rust
/// Менеджер метаданных для визуальных блоков кода
/// 
/// Отвечает за парсинг, валидацию и сохранение метаинформации
/// о расположении блоков в визуальном редакторе.
pub struct MetadataManager {
    cache: HashMap<String, Vec<BlockMetadata>>,
    validator: MetadataValidator,
}

impl MetadataManager {
    /// Создает новый менеджер метаданных
    pub fn new() -> Self {
        log::debug!("Инициализация менеджера метаданных");
        Self {
            cache: HashMap::new(),
            validator: MetadataValidator::new(),
        }
    }
    
    /// Извлекает метаданные из исходного кода
    /// 
    /// # Аргументы
    /// * `file_path` - Путь к файлу для анализа
    /// * `content` - Содержимое файла
    /// 
    /// # Возвращает
    /// Вектор найденных метаданных или ошибку парсинга
    pub async fn extract_metadata(
        &mut self, 
        file_path: &str, 
        content: &str
    ) -> Result<Vec<BlockMetadata>, MetadataError> {
        // Проверяем входные данные
        if file_path.is_empty() {
            return Err(MetadataError::EmptyFilePath);
        }
        
        if content.is_empty() {
            log::warn!("Файл {} пустой, метаданные не найдены", file_path);
            return Ok(Vec::new());
        }
        
        // Проверяем кэш
        if let Some(cached) = self.cache.get(file_path) {
            log::debug!("Найдены кэшированные метаданные для {}", file_path);
            return Ok(cached.clone());
        }
        
        // Парсим метаданные
        let metadata = self.parse_visual_meta_comments(content).await?;
        
        // Валидируем результат
        let validated = self.validator.validate_metadata(&metadata)?;
        
        // Кэшируем результат
        self.cache.insert(file_path.to_string(), validated.clone());
        
        log::info!("Извлечено {} метаданных из {}", validated.len(), file_path);
        Ok(validated)
    }
}
```

## 🔒 Безопасность

### Общие требования
- Валидация всех пользовательских данных
- Санитизация метаданных перед сохранением
- Изоляция выполнения плагинов
- Защита от XSS в пользовательских заметках

### WebSocket безопасность
- Аутентификация соединений
- Ограничение частоты сообщений
- Валидация структуры команд

## 🌟 Философия проекта

> "Multicode V3 стремится устранить барьер между мышлением программиста и его воплощением в коде. Мы создаём инструмент, который адаптируется к стилю мышления разработчика, а не заставляет адаптироваться к себе."

### Ключевые ценности
- **Свобода выбора** - разработчик сам решает, как работать с кодом
- **Производительность** - никаких задержек и лагов
- **Открытость** - расширяемость через плагины
- **Инновации** - мы внедряем будущее программирования уже сегодня

---

## 🎯 Roadmap

### Ближайшие цели
- [ ] Поддержка TypeScript и Python
- [ ] Система горячих клавиш
- [ ] Улучшенная система плагинов
- [ ] Коллаборативное редактирование

### Долгосрочные планы
- [ ] ИИ-ассистент для предложения структуры
- [ ] Интеграция с системами контроля версий
- [ ] Мобильная версия
- [ ] Облачная синхронизация проектов

---

**Помните: мы создаём будущее разработки программного обеспечения! 🚀**

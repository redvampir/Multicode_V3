use multicode_core::meta::{self, VisualMeta, DEFAULT_VERSION};

/// Состояние синхронизации между текстовым и визуальным представлениями.
#[derive(Debug, Clone, Default)]
pub struct SyncState {
    /// Текущие метаданные, извлечённые из текста.
    pub metas: Vec<VisualMeta>,
    /// Последняя версия текста, известная движку.
    pub code: String,
}

/// Сообщения для движка синхронизации.
#[derive(Debug, Clone)]
pub enum SyncMessage {
    /// Текст был изменён, необходимо перечитать метаданные.
    TextChanged(String),
    /// Визуальные метаданные были изменены, нужно обновить текст.
    VisualChanged(VisualMeta),
}

/// Простая реализация движка синхронизации.
#[derive(Debug, Default)]
pub struct SyncEngine {
    state: SyncState,
}

impl SyncEngine {
    /// Создаёт новый движок синхронизации.
    pub fn new() -> Self {
        Self::default()
    }

    /// Обрабатывает входящее сообщение синхронизации.
    /// Возвращает обновлённый текст, если он был изменён.
    pub fn handle(&mut self, msg: SyncMessage) -> Option<String> {
        match msg {
            SyncMessage::TextChanged(code) => {
                self.state.metas = meta::read_all(&code);
                self.state.code = code;
                None
            }
            SyncMessage::VisualChanged(mut meta) => {
                if meta.version == 0 {
                    meta.version = DEFAULT_VERSION;
                }
                self.state.code = meta::upsert(&self.state.code, &meta);
                self.state.metas = meta::read_all(&self.state.code);
                Some(self.state.code.clone())
            }
        }
    }

    /// Возвращает текущее состояние синхронизации.
    pub fn state(&self) -> &SyncState {
        &self.state
    }
}

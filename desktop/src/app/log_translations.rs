use crate::visual::translations::Language;
use super::state::{LogEntry, LogLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogMessage {
    FileError,
    ReadError,
    FileSaved,
    SaveError,
    FileNameMissing,
    FileExists,
    FileCreated,
    CreateError,
    DirNameMissing,
    DirCreated,
    DirCreateError,
    NewNameEmpty,
    Renamed,
    RenameError,
    Deleted,
    DeleteError,
    FoundItem,
    SearchError,
    ParseError,
    GitError,
    ExportError,
    Command,
    RunError,
    BlocksUpdated,
    Raw,
}

impl LogMessage {
    pub fn level(self) -> LogLevel {
        use LogMessage::*;
        match self {
            FileError
            | ReadError
            | SaveError
            | CreateError
            | DirCreateError
            | RenameError
            | DeleteError
            | SearchError
            | ParseError
            | GitError
            | ExportError
            | RunError => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

pub fn format_log(entry: &LogEntry, lang: Language) -> String {
    use LogMessage::*;
    let arg0 = |idx: usize| entry.args.get(idx).cloned().unwrap_or_default();
    match entry.message_key {
        FileError => match lang {
            Language::English => format!("file error: {}", arg0(0)),
            Language::Russian => format!("ошибка файла: {}", arg0(0)),
        },
        ReadError => match lang {
            Language::English => format!("read error: {}", arg0(0)),
            Language::Russian => format!("ошибка чтения: {}", arg0(0)),
        },
        FileSaved => match lang {
            Language::English => "file saved".into(),
            Language::Russian => "файл сохранен".into(),
        },
        SaveError => match lang {
            Language::English => format!("save error: {}", arg0(0)),
            Language::Russian => format!("ошибка сохранения: {}", arg0(0)),
        },
        FileNameMissing => match lang {
            Language::English => "filename is not set".into(),
            Language::Russian => "имя файла не задано".into(),
        },
        FileExists => match lang {
            Language::English => format!("{} already exists", arg0(0)),
            Language::Russian => format!("{} уже существует", arg0(0)),
        },
        FileCreated => match lang {
            Language::English => format!("created {}", arg0(0)),
            Language::Russian => format!("создан {}", arg0(0)),
        },
        CreateError => match lang {
            Language::English => format!("create error: {}", arg0(0)),
            Language::Russian => format!("ошибка создания: {}", arg0(0)),
        },
        DirNameMissing => match lang {
            Language::English => "directory name not set".into(),
            Language::Russian => "имя каталога не задано".into(),
        },
        DirCreated => match lang {
            Language::English => format!("directory created {}", arg0(0)),
            Language::Russian => format!("создан каталог {}", arg0(0)),
        },
        DirCreateError => match lang {
            Language::English => format!("directory create error: {}", arg0(0)),
            Language::Russian => format!("ошибка создания каталога: {}", arg0(0)),
        },
        NewNameEmpty => match lang {
            Language::English => "new name is empty".into(),
            Language::Russian => "новое имя пустое".into(),
        },
        Renamed => match lang {
            Language::English => format!("renamed to {}", arg0(0)),
            Language::Russian => format!("переименовано в {}", arg0(0)),
        },
        RenameError => match lang {
            Language::English => format!("rename error: {}", arg0(0)),
            Language::Russian => format!("ошибка переименования: {}", arg0(0)),
        },
        Deleted => match lang {
            Language::English => format!("deleted {}", arg0(0)),
            Language::Russian => format!("удален {}", arg0(0)),
        },
        DeleteError => match lang {
            Language::English => format!("delete error: {}", arg0(0)),
            Language::Russian => format!("ошибка удаления: {}", arg0(0)),
        },
        FoundItem => match lang {
            Language::English => format!("found {}", arg0(0)),
            Language::Russian => format!("найден {}", arg0(0)),
        },
        SearchError => match lang {
            Language::English => format!("search error: {}", arg0(0)),
            Language::Russian => format!("ошибка поиска: {}", arg0(0)),
        },
        ParseError => match lang {
            Language::English => format!("parse error: {}", arg0(0)),
            Language::Russian => format!("ошибка разбора: {}", arg0(0)),
        },
        GitError => match lang {
            Language::English => format!("git error: {}", arg0(0)),
            Language::Russian => format!("ошибка git: {}", arg0(0)),
        },
        ExportError => match lang {
            Language::English => format!("export error: {}", arg0(0)),
            Language::Russian => format!("ошибка экспорта: {}", arg0(0)),
        },
        Command => match lang {
            Language::English | Language::Russian => format!("$ {}", arg0(0)),
        },
        RunError => match lang {
            Language::English => format!("run error: {}", arg0(0)),
            Language::Russian => format!("ошибка запуска: {}", arg0(0)),
        },
        BlocksUpdated => match lang {
            Language::English => format!("blocks updated: {}", arg0(0)),
            Language::Russian => format!("обновлено блоков: {}", arg0(0)),
        },
        Raw => arg0(0),
    }
}


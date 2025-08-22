use super::state::{LogEntry, LogLevel};
use crate::visual::translations::Language;

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
            FileError | ReadError | SaveError | CreateError | DirCreateError | RenameError
            | DeleteError | SearchError | ParseError | GitError | ExportError | RunError => {
                LogLevel::Error
            }
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
            Language::Spanish => format!("error de archivo: {}", arg0(0)),
            Language::German => format!("Dateifehler: {}", arg0(0)),
        },
        ReadError => match lang {
            Language::English => format!("read error: {}", arg0(0)),
            Language::Russian => format!("ошибка чтения: {}", arg0(0)),
            Language::Spanish => format!("error de lectura: {}", arg0(0)),
            Language::German => format!("Lesefehler: {}", arg0(0)),
        },
        FileSaved => match lang {
            Language::English => "file saved".into(),
            Language::Russian => "файл сохранен".into(),
            Language::Spanish => "archivo guardado".into(),
            Language::German => "Datei gespeichert".into(),
        },
        SaveError => match lang {
            Language::English => format!("save error: {}", arg0(0)),
            Language::Russian => format!("ошибка сохранения: {}", arg0(0)),
            Language::Spanish => format!("error al guardar: {}", arg0(0)),
            Language::German => format!("Fehler beim Speichern: {}", arg0(0)),
        },
        FileNameMissing => match lang {
            Language::English => "filename is not set".into(),
            Language::Russian => "имя файла не задано".into(),
            Language::Spanish => "nombre de archivo no establecido".into(),
            Language::German => "Dateiname nicht festgelegt".into(),
        },
        FileExists => match lang {
            Language::English => format!("{} already exists", arg0(0)),
            Language::Russian => format!("{} уже существует", arg0(0)),
            Language::Spanish => format!("{} ya existe", arg0(0)),
            Language::German => format!("{} existiert bereits", arg0(0)),
        },
        FileCreated => match lang {
            Language::English => format!("created {}", arg0(0)),
            Language::Russian => format!("создан {}", arg0(0)),
            Language::Spanish => format!("creado {}", arg0(0)),
            Language::German => format!("{} erstellt", arg0(0)),
        },
        CreateError => match lang {
            Language::English => format!("create error: {}", arg0(0)),
            Language::Russian => format!("ошибка создания: {}", arg0(0)),
            Language::Spanish => format!("error de creación: {}", arg0(0)),
            Language::German => format!("Fehler beim Erstellen: {}", arg0(0)),
        },
        DirNameMissing => match lang {
            Language::English => "directory name not set".into(),
            Language::Russian => "имя каталога не задано".into(),
            Language::Spanish => "nombre de directorio no establecido".into(),
            Language::German => "Verzeichnisname nicht festgelegt".into(),
        },
        DirCreated => match lang {
            Language::English => format!("directory created {}", arg0(0)),
            Language::Russian => format!("создан каталог {}", arg0(0)),
            Language::Spanish => format!("directorio creado {}", arg0(0)),
            Language::German => format!("Verzeichnis {} erstellt", arg0(0)),
        },
        DirCreateError => match lang {
            Language::English => format!("directory create error: {}", arg0(0)),
            Language::Russian => format!("ошибка создания каталога: {}", arg0(0)),
            Language::Spanish => format!("error al crear directorio: {}", arg0(0)),
            Language::German => format!("Fehler beim Erstellen des Verzeichnisses: {}", arg0(0)),
        },
        NewNameEmpty => match lang {
            Language::English => "new name is empty".into(),
            Language::Russian => "новое имя пустое".into(),
            Language::Spanish => "el nuevo nombre está vacío".into(),
            Language::German => "neuer Name ist leer".into(),
        },
        Renamed => match lang {
            Language::English => format!("renamed to {}", arg0(0)),
            Language::Russian => format!("переименовано в {}", arg0(0)),
            Language::Spanish => format!("renombrado a {}", arg0(0)),
            Language::German => format!("umbenannt in {}", arg0(0)),
        },
        RenameError => match lang {
            Language::English => format!("rename error: {}", arg0(0)),
            Language::Russian => format!("ошибка переименования: {}", arg0(0)),
            Language::Spanish => format!("error al renombrar: {}", arg0(0)),
            Language::German => format!("Fehler beim Umbenennen: {}", arg0(0)),
        },
        Deleted => match lang {
            Language::English => format!("deleted {}", arg0(0)),
            Language::Russian => format!("удален {}", arg0(0)),
            Language::Spanish => format!("eliminado {}", arg0(0)),
            Language::German => format!("{} gelöscht", arg0(0)),
        },
        DeleteError => match lang {
            Language::English => format!("delete error: {}", arg0(0)),
            Language::Russian => format!("ошибка удаления: {}", arg0(0)),
            Language::Spanish => format!("error al eliminar: {}", arg0(0)),
            Language::German => format!("Fehler beim Löschen: {}", arg0(0)),
        },
        FoundItem => match lang {
            Language::English => format!("found {}", arg0(0)),
            Language::Russian => format!("найден {}", arg0(0)),
            Language::Spanish => format!("encontrado {}", arg0(0)),
            Language::German => format!("{} gefunden", arg0(0)),
        },
        SearchError => match lang {
            Language::English => format!("search error: {}", arg0(0)),
            Language::Russian => format!("ошибка поиска: {}", arg0(0)),
            Language::Spanish => format!("error de búsqueda: {}", arg0(0)),
            Language::German => format!("Fehler bei der Suche: {}", arg0(0)),
        },
        ParseError => match lang {
            Language::English => format!("parse error: {}", arg0(0)),
            Language::Russian => format!("ошибка разбора: {}", arg0(0)),
            Language::Spanish => format!("error de análisis: {}", arg0(0)),
            Language::German => format!("Fehler beim Parsen: {}", arg0(0)),
        },
        GitError => match lang {
            Language::English => format!("git error: {}", arg0(0)),
            Language::Russian => format!("ошибка git: {}", arg0(0)),
            Language::Spanish => format!("error de git: {}", arg0(0)),
            Language::German => format!("Git-Fehler: {}", arg0(0)),
        },
        ExportError => match lang {
            Language::English => format!("export error: {}", arg0(0)),
            Language::Russian => format!("ошибка экспорта: {}", arg0(0)),
            Language::Spanish => format!("error de exportación: {}", arg0(0)),
            Language::German => format!("Exportfehler: {}", arg0(0)),
        },
        Command => format!("$ {}", arg0(0)),
        RunError => match lang {
            Language::English => format!("run error: {}", arg0(0)),
            Language::Russian => format!("ошибка запуска: {}", arg0(0)),
            Language::Spanish => format!("error de ejecución: {}", arg0(0)),
            Language::German => format!("Fehler beim Ausführen: {}", arg0(0)),
        },
        BlocksUpdated => match lang {
            Language::English => format!("blocks updated: {}", arg0(0)),
            Language::Russian => format!("обновлено блоков: {}", arg0(0)),
            Language::Spanish => format!("bloques actualizados: {}", arg0(0)),
            Language::German => format!("Blöcke aktualisiert: {}", arg0(0)),
        },
        Raw => arg0(0),
    }
}

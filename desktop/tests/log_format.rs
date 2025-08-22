use chrono::{TimeZone, Utc};
use desktop::app::{format_log, Language, LogEntry, LogMessage};

struct Case<'a> {
    key: LogMessage,
    args: Vec<&'a str>,
    en: &'a str,
    ru: &'a str,
}

#[test]
fn format_log_translations() {
    let ts = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let cases = vec![
        Case {
            key: LogMessage::FileError,
            args: vec!["io"],
            en: "file error: io",
            ru: "ошибка файла: io",
        },
        Case {
            key: LogMessage::ReadError,
            args: vec!["io"],
            en: "read error: io",
            ru: "ошибка чтения: io",
        },
        Case {
            key: LogMessage::FileSaved,
            args: vec![],
            en: "file saved",
            ru: "файл сохранен",
        },
        Case {
            key: LogMessage::SaveError,
            args: vec!["io"],
            en: "save error: io",
            ru: "ошибка сохранения: io",
        },
        Case {
            key: LogMessage::FileNameMissing,
            args: vec![],
            en: "filename is not set",
            ru: "имя файла не задано",
        },
        Case {
            key: LogMessage::FileExists,
            args: vec!["file"],
            en: "file already exists",
            ru: "file уже существует",
        },
        Case {
            key: LogMessage::FileCreated,
            args: vec!["file"],
            en: "created file",
            ru: "создан file",
        },
        Case {
            key: LogMessage::CreateError,
            args: vec!["io"],
            en: "create error: io",
            ru: "ошибка создания: io",
        },
        Case {
            key: LogMessage::DirNameMissing,
            args: vec![],
            en: "directory name not set",
            ru: "имя каталога не задано",
        },
        Case {
            key: LogMessage::DirCreated,
            args: vec!["dir"],
            en: "directory created dir",
            ru: "создан каталог dir",
        },
        Case {
            key: LogMessage::DirCreateError,
            args: vec!["io"],
            en: "directory create error: io",
            ru: "ошибка создания каталога: io",
        },
        Case {
            key: LogMessage::NewNameEmpty,
            args: vec![],
            en: "new name is empty",
            ru: "новое имя пустое",
        },
        Case {
            key: LogMessage::Renamed,
            args: vec!["file"],
            en: "renamed to file",
            ru: "переименовано в file",
        },
        Case {
            key: LogMessage::RenameError,
            args: vec!["io"],
            en: "rename error: io",
            ru: "ошибка переименования: io",
        },
        Case {
            key: LogMessage::Deleted,
            args: vec!["file"],
            en: "deleted file",
            ru: "удален file",
        },
        Case {
            key: LogMessage::DeleteError,
            args: vec!["io"],
            en: "delete error: io",
            ru: "ошибка удаления: io",
        },
        Case {
            key: LogMessage::FoundItem,
            args: vec!["file"],
            en: "found file",
            ru: "найден file",
        },
        Case {
            key: LogMessage::SearchError,
            args: vec!["io"],
            en: "search error: io",
            ru: "ошибка поиска: io",
        },
        Case {
            key: LogMessage::ParseError,
            args: vec!["io"],
            en: "parse error: io",
            ru: "ошибка разбора: io",
        },
        Case {
            key: LogMessage::GitError,
            args: vec!["io"],
            en: "git error: io",
            ru: "ошибка git: io",
        },
        Case {
            key: LogMessage::ExportError,
            args: vec!["io"],
            en: "export error: io",
            ru: "ошибка экспорта: io",
        },
        Case {
            key: LogMessage::Command,
            args: vec!["ls"],
            en: "$ ls",
            ru: "$ ls",
        },
        Case {
            key: LogMessage::RunError,
            args: vec!["io"],
            en: "run error: io",
            ru: "ошибка запуска: io",
        },
        Case {
            key: LogMessage::BlocksUpdated,
            args: vec!["3"],
            en: "blocks updated: 3",
            ru: "обновлено блоков: 3",
        },
        Case {
            key: LogMessage::Raw,
            args: vec!["raw"],
            en: "raw",
            ru: "raw",
        },
    ];

    for case in cases {
        let entry = LogEntry::new(
            case.key,
            case.args.iter().map(|s| s.to_string()).collect(),
            ts,
        );
        assert_eq!(
            format!("[00:00:00] {}", case.en),
            format_log(&entry, Language::English),
            "{:?} English",
            case.key
        );
        assert_eq!(
            format!("[00:00:00] {}", case.ru),
            format_log(&entry, Language::Russian),
            "{:?} Russian",
            case.key
        );
    }
}

use std::collections::HashMap;

use once_cell::sync::Lazy;

use super::command_palette::CommandItem;
use super::Language;

type Translation = (&'static str, &'static str);

static COMMAND_TRANSLATIONS: Lazy<HashMap<(&'static str, Language), Translation>> =
    Lazy::new(|| {
        use Language::*;
        let mut m = HashMap::new();
        m.insert(
            ("open_file", English),
            ("Open File", "Open a file from disk"),
        );
        m.insert(
            ("open_file", Russian),
            ("Открыть файл", "Открыть файл с диска"),
        );
        m.insert(
            ("save_file", English),
            ("Save File", "Save the current file"),
        );
        m.insert(
            ("save_file", Russian),
            ("Сохранить файл", "Сохранить текущий файл"),
        );
        m.insert(
            ("toggle_terminal", English),
            ("Toggle Terminal", "Show or hide the terminal"),
        );
        m.insert(
            ("toggle_terminal", Russian),
            ("Показать/Скрыть терминал", "Показать или скрыть терминал"),
        );
        m.insert(
            ("goto_line", English),
            ("Go to Line", "Jump to specified line number"),
        );
        m.insert(
            ("goto_line", Russian),
            ("Перейти к строке", "Перейти к указанной строке"),
        );
        m.insert(
            ("open_settings", English),
            ("Open Settings", "Open application settings"),
        );
        m.insert(
            ("open_settings", Russian),
            ("Открыть настройки", "Открыть настройки приложения"),
        );
        m.insert(
            ("switch_to_text_editor", English),
            ("Switch to Text", "Switch to text editor"),
        );
        m.insert(
            ("switch_to_text_editor", Russian),
            (
                "Переключиться в текстовый редактор",
                "Переключиться в текстовый редактор",
            ),
        );
        m.insert(
            ("switch_to_visual_editor", English),
            ("Switch to Visual", "Switch to visual editor"),
        );
        m.insert(
            ("switch_to_visual_editor", Russian),
            (
                "Переключиться в визуальный редактор",
                "Переключиться в визуальный редактор",
            ),
        );
        m.insert(
            ("switch_to_split", English),
            ("Switch to Split", "Switch to split view"),
        );
        m.insert(
            ("switch_to_split", Russian),
            (
                "Переключиться в режим разделения",
                "Переключиться в режим разделения",
            ),
        );
        m
    });

pub fn command_name(cmd: &CommandItem, lang: Language) -> &'static str {
    COMMAND_TRANSLATIONS
        .get(&(cmd.id, lang))
        .or_else(|| COMMAND_TRANSLATIONS.get(&(cmd.id, Language::English)))
        .map(|(name, _)| *name)
        .unwrap_or(cmd.id)
}

pub fn command_description(cmd: &CommandItem, lang: Language) -> &'static str {
    COMMAND_TRANSLATIONS
        .get(&(cmd.id, lang))
        .or_else(|| COMMAND_TRANSLATIONS.get(&(cmd.id, Language::English)))
        .map(|(_, desc)| *desc)
        .unwrap_or("")
}

pub fn command_hotkey(cmd: &CommandItem, lang: Language) -> String {
    format_hotkey(cmd.hotkey, lang)
}

fn format_hotkey(raw: &str, lang: Language) -> String {
    raw.split('+')
        .map(|part| translate_part(part, lang))
        .collect::<Vec<_>>()
        .join("+")
}

fn translate_part(part: &str, lang: Language) -> String {
    match part {
        "Ctrl" => ctrl_label(),
        "Alt" => alt_label(),
        "Shift" => "Shift".into(),
        key => translate_key(key, lang),
    }
}

fn ctrl_label() -> String {
    #[cfg(target_os = "macos")]
    {
        "Cmd".into()
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl".into()
    }
}

fn alt_label() -> String {
    #[cfg(target_os = "macos")]
    {
        "Option".into()
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Alt".into()
    }
}

fn translate_key(key: &str, lang: Language) -> String {
    if lang != Language::Russian {
        return key.to_string();
    }
    match key {
        "`" => "Ё".into(),
        "," => "Б".into(),
        "." => "Ю".into(),
        ";" => "Ж".into(),
        "'" => "Э".into(),
        "[" => "Х".into(),
        "]" => "Ъ".into(),
        _ => match key.to_ascii_uppercase().as_str() {
            "Q" => "Й".into(),
            "W" => "Ц".into(),
            "E" => "У".into(),
            "R" => "К".into(),
            "T" => "Е".into(),
            "Y" => "Н".into(),
            "U" => "Г".into(),
            "I" => "Ш".into(),
            "O" => "Щ".into(),
            "P" => "З".into(),
            "A" => "Ф".into(),
            "S" => "Ы".into(),
            "D" => "В".into(),
            "F" => "А".into(),
            "G" => "П".into(),
            "H" => "Р".into(),
            "J" => "О".into(),
            "K" => "Л".into(),
            "L" => "Д".into(),
            "Z" => "Я".into(),
            "X" => "Ч".into(),
            "C" => "С".into(),
            "V" => "М".into(),
            "B" => "И".into(),
            "N" => "Т".into(),
            "M" => "Ь".into(),
            _ => key.to_string(),
        },
    }
}

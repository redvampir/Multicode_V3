use super::command_palette::CommandItem;
use super::Language;

pub fn command_name(cmd: &CommandItem, lang: Language) -> &'static str {
    match lang {
        Language::Russian => cmd.name_ru,
        _ => cmd.name_en,
    }
}

pub fn command_description(cmd: &CommandItem, lang: Language) -> &'static str {
    match lang {
        Language::Russian => cmd.description_ru,
        _ => cmd.description_en,
    }
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

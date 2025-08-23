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

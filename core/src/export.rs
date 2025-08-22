use crate::meta;
use once_cell::sync::Lazy;
use regex::{Error as RegexError, Regex};
use serde::{Deserialize, Serialize};

/// Набор визуальных метаданных, связанных с исходным файлом.
#[derive(Debug, Serialize, Deserialize)]
pub struct VizDocument {
    /// Записи визуальных метаданных, извлечённые из файла.
    pub nodes: Vec<meta::VisualMeta>,
}

/// Сериализует все комментарии `@VISUAL_META` из `content` в JSON‑строку `VizDocument`.
pub fn serialize_viz_document(content: &str) -> Option<String> {
    let metas = meta::read_all(content);
    if metas.is_empty() {
        None
    } else {
        serde_json::to_string(&VizDocument { nodes: metas }).ok()
    }
}

/// Десериализует `VizDocument` из JSON‑строки.
pub fn deserialize_viz_document(json: &str) -> Result<VizDocument, serde_json::Error> {
    serde_json::from_str(json)
}

// Регулярные выражения для разных стилей комментариев, которые могут содержать
// маркеры `@VISUAL_META`. Каждый шаблон также поглощает завершающий перевод строки,
// чтобы убрать всю строку из вывода.
static PYTHON_SINGLE: Lazy<Result<Regex, RegexError>> =
    Lazy::new(|| Regex::new(r"(?m)^\s*#\s*@VISUAL_META\s*\{.*\}\s*\n?"));

static SLASH_SINGLE: Lazy<Result<Regex, RegexError>> =
    Lazy::new(|| Regex::new(r"(?m)^\s*//\s*@VISUAL_META\s*\{.*\}\s*\n?"));

static C_STYLE_MULTI: Lazy<Result<Regex, RegexError>> =
    Lazy::new(|| Regex::new(r"(?ms)^\s*/\*\s*@VISUAL_META\s*\{.*?\}\s*\*/\s*\n?"));

static HTML_MULTI: Lazy<Result<Regex, RegexError>> =
    Lazy::new(|| Regex::new(r"(?ms)^\s*<!--\s*@VISUAL_META\s*\{.*?\}\s*-->\s*\n?"));

/// Удаляет из `content` все строки, содержащие комментарии `@VISUAL_META`.
///
/// Поддерживаются однострочные комментарии, начинающиеся с `#` или `//`, и
/// блочные комментарии вида `/* */` и `<!-- -->`.
pub fn remove_meta_lines(content: &str) -> Result<String, RegexError> {
    let mut out = content.to_string();
    for re in [
        &*PYTHON_SINGLE,
        &*SLASH_SINGLE,
        &*C_STYLE_MULTI,
        &*HTML_MULTI,
    ] {
        let re = re.as_ref().map_err(|e| e.clone())?;
        out = re.replace_all(&out, "").to_string();
    }
    Ok(out)
}

/// Подготавливает исходный текст к экспорту.
///
/// Если `strip_meta` равно `true`, все комментарии `@VISUAL_META` удаляются.
/// Иначе содержимое возвращается без изменений.
pub fn prepare_for_export(content: &str, strip_meta: bool) -> Result<String, RegexError> {
    if strip_meta {
        remove_meta_lines(content)
    } else {
        Ok(content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_python_comment() {
        let src = "# @VISUAL_META {\"id\":1}\nprint(\"hi\")\n";
        let cleaned = remove_meta_lines(src).unwrap();
        assert!(!cleaned.contains("@VISUAL_META"));
        assert!(cleaned.contains("print"));
    }

    #[test]
    fn serialize_empty_returns_none() {
        assert!(serialize_viz_document("").is_none());
    }

    #[test]
    fn deserialize_invalid_json() {
        assert!(deserialize_viz_document("not json").is_err());
    }
}

use std::collections::HashMap;
use std::fs;
use std::sync::OnceLock;

/// Internal type for storing translations.
type Map = HashMap<String, HashMap<String, String>>;

/// Global storage for translations loaded from JSON.
static TRANSLATIONS: OnceLock<Map> = OnceLock::new();

fn translations() -> &'static Map {
    TRANSLATIONS.get_or_init(|| {
        serde_json::from_str(include_str!("translations.json"))
            .expect("invalid default translations")
    })
}

/// Load translations from a JSON file at runtime.
pub fn load_from_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = fs::read_to_string(path)?;
    let map: Map = serde_json::from_str(&data)?;
    TRANSLATIONS
        .set(map)
        .map_err(|_| "translations already loaded".into())
}

/// Возвращает стандартные переводы для известных типов блоков.
pub fn lookup(kind: &str) -> Option<HashMap<String, String>> {
    translations().get(kind).cloned()
}

/// Список доступных языков в текущих переводах.
pub fn languages() -> Vec<String> {
    translations()
        .values()
        .next()
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

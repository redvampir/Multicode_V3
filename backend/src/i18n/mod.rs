use crate::meta::Translations;

/// Return default translations for known block kinds.
pub fn lookup(kind: &str) -> Option<Translations> {
    match kind {
        "Function" => Some(Translations {
            ru: Some("Функция".into()),
            en: Some("Function".into()),
            es: Some("Función".into()),
        }),
        "Variable" => Some(Translations {
            ru: Some("Переменная".into()),
            en: Some("Variable".into()),
            es: Some("Variable".into()),
        }),
        "Condition" => Some(Translations {
            ru: Some("Условие".into()),
            en: Some("Condition".into()),
            es: Some("Condición".into()),
        }),
        "Loop" => Some(Translations {
            ru: Some("Цикл".into()),
            en: Some("Loop".into()),
            es: Some("Bucle".into()),
        }),
        _ => None,
    }
}

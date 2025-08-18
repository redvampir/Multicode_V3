use std::collections::HashMap;

/// Возвращает стандартные переводы для известных типов блоков.
pub fn lookup(kind: &str) -> Option<HashMap<String, String>> {
    let mut map = HashMap::new();
    match kind {
        "Function" => {
            map.insert("ru".into(), "Функция".into());
            map.insert("en".into(), "Function".into());
            map.insert("es".into(), "Función".into());
            Some(map)
        }
        "Variable" => {
            map.insert("ru".into(), "Переменная".into());
            map.insert("en".into(), "Variable".into());
            map.insert("es".into(), "Variable".into());
            Some(map)
        }
        "Condition" => {
            map.insert("ru".into(), "Условие".into());
            map.insert("en".into(), "Condition".into());
            map.insert("es".into(), "Condición".into());
            Some(map)
        }
        "Loop" => {
            map.insert("ru".into(), "Цикл".into());
            map.insert("en".into(), "Loop".into());
            map.insert("es".into(), "Bucle".into());
            Some(map)
        }
        _ => None,
    }
}

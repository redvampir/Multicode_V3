use crate::parser::viz_comments::{load_viz_document, parse_viz_comments, VizDocument};
use std::collections::HashSet;
use std::path::Path;

/// Список разрешённых названий операций. Это минимальный набор, используемый в
/// тестах, и его можно расширять по мере добавления новых визуальных блоков.
const ALLOWED_OPS: &[&str] = &["inc", "dec", "Add", "ternary"];

/// Проверяет исходный текст, содержащий комментарии `@viz`.
///
/// Возвращает список понятных человеку сообщений об ошибках. Пустой список
/// означает, что проблемы не обнаружены.
pub fn lint_str(content: &str) -> Vec<String> {
    let doc = parse_viz_comments(content);
    lint_document(&doc)
}

/// Проверяет файл на диске, используя соседний документ `*.viz.json` или
/// встроенные комментарии `@viz`.
///
/// Функция возвращает список обнаруженных проблем или ошибку ввода-вывода,
/// если файл не удалось прочитать.
pub fn lint_file(path: &Path) -> std::io::Result<Vec<String>> {
    let doc = load_viz_document(path)?;
    Ok(lint_document(&doc))
}

/// Выполняет проверку [`VizDocument`].
fn lint_document(doc: &VizDocument) -> Vec<String> {
    let mut errors = Vec::new();
    let mut node_ids = HashSet::new();
    let known_ids: HashSet<&str> = doc
        .nodes
        .iter()
        .filter_map(|n| n.id.as_deref())
        .collect();

    for entry in &doc.nodes {
        let ident = entry.id.as_deref().unwrap_or("<unknown>");

        match entry.op.as_deref() {
            Some(op) if ALLOWED_OPS.contains(&op) => {}
            Some(op) => errors.push(format!("узел {ident}: неизвестная операция `{op}`")),
            None => errors.push(format!("узел {ident}: отсутствует операция")),
        }

        match entry.node.as_deref() {
            Some(n) => {
                if n.parse::<u32>().is_err() {
                    errors.push(format!("узел {ident}: некорректный node `{n}`"));
                }
                if !node_ids.insert(n) {
                    errors.push(format!("дублирующийся идентификатор узла `{n}`"));
                }
            }
            None => errors.push(format!("узел {ident}: отсутствует node")),
        }

        for inp in &entry.inputs {
            if !known_ids.contains(inp.as_str()) {
                errors.push(format!("узел {ident}: неизвестный вход `{inp}`"));
            }
        }
        for out in &entry.outputs {
            if !known_ids.contains(out.as_str()) {
                errors.push(format!("узел {ident}: неизвестный выход `{out}`"));
            }
        }
    }

    errors
}

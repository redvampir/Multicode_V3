use crate::meta::VisualMeta;
use serde_json::Value;

/// Логическое выражение запроса для фильтрации записей [`VisualMeta`].
#[derive(Debug, Clone)]
pub enum Expr {
    /// Все подвыражения должны совпасть.
    And(Vec<Expr>),
    /// Достаточно совпадения любого подвыражения.
    Or(Vec<Expr>),
    /// Поле содержит значение (с учётом регистра).
    Cond { field: String, value: String },
}

/// Разбирает простые выражения `AND`/`OR`, состоящие из условий `field:value`.
/// Примеры: `id:foo AND tags:bar`, `id:foo OR id:bar`.
pub fn parse(input: &str) -> Expr {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    parse_or(&tokens, 0).0
}

fn parse_or(tokens: &[&str], pos: usize) -> (Expr, usize) {
    let (mut expr, mut i) = parse_and(tokens, pos);
    while i < tokens.len() {
        if tokens[i].eq_ignore_ascii_case("OR") {
            let (rhs, j) = parse_and(tokens, i + 1);
            expr = Expr::Or(vec![expr, rhs]);
            i = j;
        } else {
            break;
        }
    }
    (expr, i)
}

fn parse_and(tokens: &[&str], pos: usize) -> (Expr, usize) {
    let (expr, mut i) = parse_term(tokens, pos);
    let mut terms = vec![expr];
    while i < tokens.len() {
        if tokens[i].eq_ignore_ascii_case("AND") {
            let (rhs, j) = parse_term(tokens, i + 1);
            terms.push(rhs);
            i = j;
        } else if tokens[i].eq_ignore_ascii_case("OR") {
            break;
        } else {
            // неявное AND
            let (rhs, j) = parse_term(tokens, i);
            terms.push(rhs);
            i = j;
        }
    }
    if terms.len() == 1 {
        (terms.remove(0), i)
    } else {
        (Expr::And(terms), i)
    }
}

fn parse_term(tokens: &[&str], pos: usize) -> (Expr, usize) {
    if pos >= tokens.len() {
        return (Expr::And(vec![]), pos);
    }
    let tok = tokens[pos];
    if let Some(idx) = tok.find(':') {
        let field = tok[..idx].to_string();
        let value = tok[idx + 1..].to_string();
        (Expr::Cond { field, value }, pos + 1)
    } else {
        // трактуем отдельное слово как поиск значения по любому полю
        (
            Expr::Cond {
                field: String::from("*"),
                value: tok.to_string(),
            },
            pos + 1,
        )
    }
}

/// Проверяет, удовлетворяет ли [`VisualMeta`] выражению.
pub fn matches(meta: &VisualMeta, expr: &Expr) -> bool {
    match expr {
        Expr::And(list) => list.iter().all(|e| matches(meta, e)),
        Expr::Or(list) => list.iter().any(|e| matches(meta, e)),
        Expr::Cond { field, value } => {
            let val = serde_json::to_value(meta).unwrap_or(Value::Null);
            match_field(&val, field, value)
        }
    }
}

fn match_field(val: &Value, field: &str, target: &str) -> bool {
    if field == "*" {
        return val.to_string().contains(target);
    }
    match val.get(field) {
        Some(Value::String(s)) => s.contains(target),
        Some(Value::Array(arr)) => arr.iter().any(|v| match v {
            Value::String(s) => s.contains(target),
            _ => false,
        }),
        Some(v) => v.to_string().contains(target),
        None => false,
    }
}

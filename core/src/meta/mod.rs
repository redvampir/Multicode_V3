use chrono::Utc;
use once_cell::sync::Lazy;
use serde::Serialize;
use tracing::error;
use std::collections::HashSet;
use std::sync::Mutex;
mod comment_detector;
#[cfg(feature = "db")]
pub mod db;
pub mod id_registry;
pub mod query;
mod types;
#[cfg(feature = "watch")]
pub mod watch;
pub use types::{AiNote, VisualMeta, DEFAULT_VERSION};

/// Маркер, используемый для идентификации комментариев с визуальными метаданными в документах.
const MARKER: &str = "@VISUAL_META";

/// Мьютекс, используемый для сериализации доступа к глобальному [`id_registry`].
///
/// Такие функции, как [`read_all`], модифицируют реестр, очищая и заполняя его
/// заново, что может приводить к гонкам при вызове из нескольких потоков.
/// Тесты выполняются параллельно и раньше могли перемешивать эти операции,
/// оставляя реестр в неконсистентном состоянии и вызывая ошибки поиска.
/// Блокируя весь `read_all`, мы обеспечиваем эксклюзивный доступ к реестру.
static REGISTRY_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn migrate(meta: &mut VisualMeta) {
    if meta.version < DEFAULT_VERSION {
        meta.version = DEFAULT_VERSION;
    }
}

/// Структурированная ошибка валидации для [`VisualMeta`].
#[derive(Debug, Serialize)]
pub struct ValidationError {
    /// Поле, в котором произошла ошибка валидации.
    pub field: String,
    /// Описание проблемы валидации.
    pub message: String,
}

/// Валидирует экземпляр [`VisualMeta`].
pub fn validate(meta: &VisualMeta) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    if meta.id.trim().is_empty() {
        errors.push(ValidationError {
            field: "id".into(),
            message: "id не должен быть пустым".into(),
        });
    }

    if !meta.x.is_finite() {
        errors.push(ValidationError {
            field: "x".into(),
            message: "x должен быть конечным числом".into(),
        });
    }

    if !meta.y.is_finite() {
        errors.push(ValidationError {
            field: "y".into(),
            message: "y должен быть конечным числом".into(),
        });
    }

    let mut tag_set = HashSet::new();
    for tag in &meta.tags {
        if !tag_set.insert(tag) {
            errors.push(ValidationError {
                field: "tags".into(),
                message: format!("дублирующийся тег '{tag}'"),
            });
        }
    }

    let mut link_set = HashSet::new();
    for link in &meta.links {
        if !link_set.insert(link) {
            errors.push(ValidationError {
                field: "links".into(),
                message: format!("дублирующаяся ссылка '{link}'"),
            });
        }
    }

    let mut anchor_set = HashSet::new();
    for anc in &meta.anchors {
        if !anchor_set.insert(anc) {
            errors.push(ValidationError {
                field: "anchors".into(),
                message: format!("дублирующийся anchor '{anc}'"),
            });
        }
    }

    if let Some(ext) = &meta.extends {
        if ext.trim().is_empty() {
            errors.push(ValidationError {
                field: "extends".into(),
            message: "extends не должен быть пустым".into(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Вставляет или обновляет комментарий с визуальными метаданными в `content`.
///
/// Если комментария ещё нет, он будет помещён в начало документа.
/// `preserve_formatting` сохраняет исходные отступы и суффикс строки
/// существующего комментария.
pub fn upsert(content: &str, meta: &VisualMeta, preserve_formatting: bool) -> String {
    let marker = format!("<!-- {} ", MARKER);
    let mut meta = meta.clone();
    meta.updated_at = Utc::now();
    if let Err(errs) = validate(&meta) {
        error!("невалидный VisualMeta: {:?}", errs);
        return content.to_string();
    }
    let serialized = match serde_json::to_string(&meta) {
        Ok(s) => s,
        Err(e) => {
            error!("не удалось сериализовать VisualMeta: {e}");
            return content.to_string();
        }
    };

    let mut out = String::new();
    let mut found = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(&marker) {
            if let Some(end_idx) = trimmed.find("-->") {
                let json_part = &trimmed[marker.len()..end_idx].trim();
                if let Ok(existing) = serde_json::from_str::<VisualMeta>(json_part) {
                    if existing.id == meta.id {
                        let prefix = if preserve_formatting {
                            &line[..line.len() - trimmed.len()]
                        } else {
                            ""
                        };
                        let suffix = if preserve_formatting {
                            &trimmed[end_idx + 3..]
                        } else {
                            ""
                        };
                        out.push_str(prefix);
                        out.push_str(&format!("{}{} -->", marker, serialized));
                        if preserve_formatting {
                            out.push_str(suffix);
                        }
                        out.push('\n');
                        found = true;
                        continue;
                    }
                }
            }
        }
        out.push_str(line);
        out.push('\n');
    }

    if !found {
        out = format!("{}{} -->\n{}", marker, serialized, out);
    }

    out
}

/// Считывает все комментарии с визуальными метаданными из `content`.
///
/// Возвращает разобранные метаданные и список дублирующихся идентификаторов.
pub fn read_all_with_dups(content: &str) -> (Vec<VisualMeta>, Vec<String>) {
    // Сериализуем доступ, чтобы реестр не очищался, пока другой поток
    // его использует. Иначе это могло приводить к пропущенным метаданным
    // и нестабильным тестам.
    let _guard = match REGISTRY_LOCK.lock() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("Registry lock poisoned: {e}");
            e.into_inner()
        }
    };
    id_registry::clear();
    let mut ids = Vec::new();
    for json in comment_detector::extract_json(content) {
        if let Ok(mut meta) = serde_json::from_str::<VisualMeta>(&json) {
            migrate(&mut meta);
            let id = meta.id.clone();
            id_registry::register(meta);
            ids.push(id);
        }
    }
    let metas = ids
        .into_iter()
        .filter_map(|id| merge_base_meta(&id))
        .collect();
    let dups = id_registry::duplicates();
    (metas, dups)
}

/// Считывает все комментарии с визуальными метаданными из `content`,
/// отбрасывая дублирующиеся идентификаторы.
pub fn read_all(content: &str) -> Vec<VisualMeta> {
    read_all_with_dups(content).0
}

/// Рекурсивно объединяет метаданные с их базовыми записями, следуя цепочке `extends`.
pub fn merge_base_meta(id: &str) -> Option<VisualMeta> {
    fn inner(id: &str, visited: &mut HashSet<String>) -> Option<VisualMeta> {
        if !visited.insert(id.to_string()) {
            return id_registry::get(id);
        }
        let mut meta = id_registry::get(id)?;
        if let Some(parent_id) = meta.extends.clone() {
            if let Some(base) = inner(&parent_id, visited) {
                meta = merge_two(base, meta);
            }
        }
        meta.extends = None;
        Some(meta)
    }

    fn merge_two(base: VisualMeta, mut child: VisualMeta) -> VisualMeta {
        child.tags =
            base.tags
                .into_iter()
                .chain(child.tags.into_iter())
                .fold(Vec::new(), |mut acc, tag| {
                    if !acc.contains(&tag) {
                        acc.push(tag);
                    }
                    acc
                });
        child.links = base.links.into_iter().chain(child.links.into_iter()).fold(
            Vec::new(),
            |mut acc, link| {
                if !acc.contains(&link) {
                    acc.push(link);
                }
                acc
            },
        );
        child.anchors = base
            .anchors
            .into_iter()
            .chain(child.anchors.into_iter())
            .fold(Vec::new(), |mut acc, anc| {
                if !acc.contains(&anc) {
                    acc.push(anc);
                }
                acc
            });
        if child.origin.is_none() {
            child.origin = base.origin;
        }
        for (k, v) in base.translations {
            child.translations.entry(k).or_insert(v);
        }
        child.ai = match (child.ai, base.ai) {
            (Some(mut c), Some(b)) => {
                if c.description.is_none() {
                    c.description = b.description;
                }
                for hint in b.hints.into_iter().rev() {
                    if !c.hints.contains(&hint) {
                        c.hints.insert(0, hint);
                    }
                }
                Some(c)
            }
            (Some(c), None) => Some(c),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        if child.extras.is_none() {
            child.extras = base.extras;
        }
        if child.updated_at.timestamp() == 0 {
            child.updated_at = base.updated_at;
        }
        child
    }

    inner(id, &mut HashSet::new())
}

/// Удаляет из `content` все комментарии с визуальными метаданными.
pub fn remove_all(content: &str) -> String {
    comment_detector::strip(content)
}

/// Удобная обёртка, возвращающая все записи метаданных из `content`.
pub fn list(content: &str) -> Vec<VisualMeta> {
    read_all(content)
}

/// Исправляет проблемы в комментариях метаданных, например дублирующиеся идентификаторы.
///
/// При обнаружении дубликатов генерируются новые уникальные значения, и
/// обновлённые комментарии снова вставляются в документ.
pub fn fix_all(content: &str) -> String {
    let mut metas = read_all(content);
    let mut seen = HashSet::new();
    let mut changed = false;
    for meta in &mut metas {
        if !seen.insert(meta.id.clone()) {
            meta.id = unique_id();
            changed = true;
        }
    }

    if !changed {
        return content.to_string();
    }

    let mut out = remove_all(content);
    for meta in &metas {
        out = upsert(&out, meta, false);
    }
    out
}

fn unique_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos(),
        Err(e) => {
            eprintln!("System time error: {e}");
            0
        }
    };
    format!("m{}", nanos)
}

#[cfg(test)]
  mod tests {
      use super::*;
      use chrono::Utc;
      use serde_json::json;
      use std::collections::HashMap;
      use tracing_test::traced_test;

    #[test]
    fn upsert_and_read_roundtrip() {
        let meta = VisualMeta {
            version: 1,
            id: "1".into(),
            x: 10.0,
            y: 20.0,
            tags: vec!["alpha".into(), "beta".into()],
            links: vec![],
            anchors: vec!["a".into()],
            tests: vec![],
            extends: None,
            origin: None,
            translations: HashMap::new(),
            ai: Some(AiNote {
                description: Some("desc".into()),
                hints: vec!["hint".into()],
            }),
            extras: Some(json!({"foo": "bar"})),
            updated_at: Utc::now(),
        };
        let content = "fn main() {}";
        let updated = upsert(content, &meta, false);
        assert!(updated.contains(MARKER));
        let metas = read_all(&updated);
        assert_eq!(metas.len(), 1);
        assert_eq!(metas[0].x, 10.0);
        assert_eq!(metas[0].tags, vec!["alpha", "beta"]);
        assert!(metas[0].links.is_empty());
        assert_eq!(metas[0].anchors, vec!["a"]);
        assert_eq!(
            metas[0].ai.as_ref().unwrap().description.as_deref(),
            Some("desc")
        );
        assert_eq!(metas[0].extras, Some(json!({"foo": "bar"})));
        assert!(metas[0].updated_at.timestamp() > 0);
        assert_eq!(metas[0].version, 1);
    }

    #[test]
    fn remove_all_strips_metadata() {
        let content = format!("line1\n<!-- {} {{\"id\":\"1\"}} -->\nline2\n", MARKER);
        let cleaned = remove_all(&content);
        assert!(!cleaned.contains(MARKER));
        assert!(cleaned.contains("line1"));
        assert!(cleaned.contains("line2"));
    }

    #[test]
    fn fix_all_replaces_duplicate_ids() {
        let content = format!(
            "<!-- {} {{\"id\":\"1\",\"x\":0.0,\"y\":0.0}} -->\n<!-- {} {{\"id\":\"1\",\"x\":1.0,\"y\":1.0}} -->",
            MARKER, MARKER
        );
        let fixed = fix_all(&content);
        let metas = read_all(&fixed);
        assert_eq!(metas.len(), 2);
        assert_ne!(metas[0].id, metas[1].id);
    }

    #[test]
    fn merge_base_meta_combines_fields() {
        let parent = VisualMeta {
            version: 1,
            id: "p".into(),
            x: 0.0,
            y: 0.0,
            tags: vec!["base".into()],
            links: vec!["l1".into()],
            anchors: vec!["a1".into()],
            tests: vec![],
            extends: None,
            origin: Some("orig".into()),
            translations: {
                let mut m = HashMap::new();
                m.insert("en".into(), "Parent".into());
                m
            },
            ai: Some(AiNote {
                description: Some("pdesc".into()),
                hints: vec!["h1".into()],
            }),
            extras: None,
            updated_at: Utc::now(),
        };

        let child = VisualMeta {
            version: 1,
            id: "c".into(),
            x: 1.0,
            y: 1.0,
            tags: vec!["child".into()],
            links: vec!["l2".into()],
            anchors: vec!["a2".into()],
            tests: vec![],
            extends: Some("p".into()),
            origin: None,
            translations: {
                let mut m = HashMap::new();
                m.insert("ru".into(), "Дитя".into());
                m
            },
            ai: Some(AiNote {
                description: None,
                hints: vec!["h2".into()],
            }),
            extras: None,
            updated_at: Utc::now(),
        };

        let content = format!(
            "// @VISUAL_META {}\n// @VISUAL_META {}\n",
            serde_json::to_string(&parent).unwrap(),
            serde_json::to_string(&child).unwrap()
        );

        let metas = read_all(&content);
        let merged = metas.iter().find(|m| m.id == "c").unwrap();
        assert_eq!(merged.tags, vec!["base", "child"]);
        assert_eq!(merged.links, vec!["l1", "l2"]);
        assert_eq!(merged.anchors, vec!["a1", "a2"]);
        assert_eq!(merged.origin.as_deref(), Some("orig"));
        assert_eq!(merged.translations.get("en").unwrap(), "Parent");
        assert_eq!(merged.translations.get("ru").unwrap(), "Дитя");
        let ai = merged.ai.as_ref().unwrap();
        assert_eq!(ai.description.as_deref(), Some("pdesc"));
        assert_eq!(ai.hints, vec!["h1", "h2"]);
    }

    #[test]
    fn read_all_with_dups_handles_invalid_json() {
        let content = format!("<!-- {} {{invalid}} -->", MARKER);
        let (metas, dups) = read_all_with_dups(&content);
        assert!(metas.is_empty());
        assert!(dups.is_empty());
    }

      #[test]
      fn validate_reports_errors() {
        let meta = VisualMeta {
            version: 1,
            id: "".into(),
            x: f64::NAN,
            y: 0.0,
            tags: vec!["a".into(), "a".into()],
            links: vec!["l".into(), "l".into()],
            anchors: vec!["a".into(), "a".into()],
            tests: vec![],
            extends: Some("".into()),
            origin: None,
            translations: HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        };
        let errs = validate(&meta).unwrap_err();
        assert!(errs.iter().any(|e| e.field == "id"));
        assert!(errs.iter().any(|e| e.field == "x"));
        assert!(errs.iter().any(|e| e.field == "tags"));
        assert!(errs.iter().any(|e| e.field == "links"));
        assert!(errs.iter().any(|e| e.field == "anchors"));
        assert!(errs.iter().any(|e| e.field == "extends"));
      }

      #[test]
      #[traced_test]
      fn upsert_logs_error_on_invalid_meta() {
          let meta = VisualMeta {
              version: 1,
              id: "".into(),
              x: f64::NAN,
              y: 0.0,
              tags: vec![],
              links: vec![],
              anchors: vec![],
              tests: vec![],
              extends: Some("".into()),
              origin: None,
              translations: HashMap::new(),
              ai: None,
              extras: None,
              updated_at: Utc::now(),
          };
          let content = "test";
          let out = upsert(content, &meta, false);
          assert_eq!(out, content);
          assert!(logs_contain("невалидный VisualMeta"));
      }
  }

use multicode_core::meta::VisualMeta;
use std::collections::BTreeSet;

/// Тип конфликта между текстовым и визуальным представлениями.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// Текстовая структура изменилась.
    Structural,
    /// Блок был перемещён на холсте.
    BlockMoved,
    /// Метаданные в комментариях расходятся.
    MetaComment,
}

/// Описание конфликта синхронизации.
///
/// Конфликт всегда связан с конкретным `id` из [`VisualMeta`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncConflict {
    /// Идентификатор метаданных, для которых возник конфликт.
    pub id: String,
    /// Тип обнаруженного конфликта.
    pub kind: ConflictType,
}

/// Варианты разрешения конфликта.
#[derive(Debug, Clone)]
pub enum ResolutionOption {
    /// Оставить данные из текстового представления.
    UseText(VisualMeta),
    /// Использовать данные из визуального представления.
    UseVisual(VisualMeta),
    /// Объединить данные из обоих источников.
    Merge(VisualMeta),
}

/// Разрешает конфликты между версиями метаданных.
pub struct ConflictResolver;

impl ConflictResolver {
    /// Выявляет тип конфликта между метаданными из текста и визуального редактора.
    fn detect(text: &VisualMeta, visual: &VisualMeta) -> ConflictType {
        if (text.x - visual.x).abs() > f64::EPSILON || (text.y - visual.y).abs() > f64::EPSILON {
            ConflictType::BlockMoved
        } else if text.tags != visual.tags
            || text.links != visual.links
            || text.anchors != visual.anchors
            || text.tests != visual.tests
        {
            ConflictType::MetaComment
        } else {
            ConflictType::Structural
        }
    }

    fn identical_except_version(a: &VisualMeta, b: &VisualMeta) -> bool {
        (a.x - b.x).abs() <= f64::EPSILON
            && (a.y - b.y).abs() <= f64::EPSILON
            && a.tags == b.tags
            && a.links == b.links
            && a.anchors == b.anchors
            && a.tests == b.tests
            && a.extends == b.extends
            && a.origin == b.origin
            && a.translations == b.translations
            && a.extras == b.extras
    }

    /// Разрешает конфликт между двумя версиями метаданных.
    pub fn resolve(text: &VisualMeta, visual: &VisualMeta) -> (ResolutionOption, SyncConflict) {
        let kind = Self::detect(text, visual);
        let conflict = SyncConflict {
            id: text.id.clone(),
            kind: kind.clone(),
        };
        let option = match kind {
            ConflictType::Structural => {
                if Self::identical_except_version(text, visual) {
                    ResolutionOption::UseVisual(visual.clone())
                } else {
                    ResolutionOption::UseText(text.clone())
                }
            }
            ConflictType::BlockMoved => ResolutionOption::UseVisual(visual.clone()),
            ConflictType::MetaComment => {
                let mut merged = text.clone();
                merged.version = text.version.max(visual.version);

                let merge_vec = |a: &[String], b: &[String]| {
                    a.iter()
                        .chain(b.iter())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .cloned()
                        .collect::<Vec<_>>()
                };

                merged.tags = merge_vec(&text.tags, &visual.tags);
                merged.links = merge_vec(&text.links, &visual.links);
                merged.anchors = merge_vec(&text.anchors, &visual.anchors);
                merged.tests = merge_vec(&text.tests, &visual.tests);

                ResolutionOption::Merge(merged)
            }
        };
        (option, conflict)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    fn meta(id: &str, x: f64, tags: Vec<&str>) -> VisualMeta {
        VisualMeta {
            version: 1,
            id: id.to_string(),
            x,
            y: 0.0,
            tags: tags.into_iter().map(|s| s.to_string()).collect(),
            links: Vec::new(),
            anchors: Vec::new(),
            tests: Vec::new(),
            extends: None,
            origin: None,
            translations: HashMap::new(),
            ai: None,
            extras: None,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn detect_block_move() {
        let a = meta("a", 0.0, vec![]);
        let b = meta("a", 1.0, vec![]);
        let (_, conflict) = ConflictResolver::resolve(&a, &b);
        assert_eq!(conflict.kind, ConflictType::BlockMoved);
    }
}

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fmt;

/// Identifier for a lesson inside the training curriculum.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LessonId(String);

impl LessonId {
    /// Create a new lesson identifier after validating the provided value.
    pub fn new(value: impl Into<String>) -> Result<Self, CurriculumError> {
        let value = value.into().trim().to_owned();
        if value.is_empty() {
            return Err(CurriculumError::EmptyIdentifier);
        }
        Ok(Self(value))
    }

    /// Expose the identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Classification of a lesson.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LessonKind {
    Theory,
    Practice,
    Assessment,
}

/// Recommended skill level for a lesson.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillLevel {
    Beginner,
    Intermediate,
    Advanced,
}

/// A single lesson in the training curriculum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lesson {
    id: LessonId,
    title: String,
    theme: String,
    kind: LessonKind,
    skill_level: SkillLevel,
    estimated_minutes: u16,
    tags: Vec<String>,
    summary: String,
}

impl Lesson {
    /// Validate input and build a new lesson instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: LessonId,
        title: impl Into<String>,
        theme: impl Into<String>,
        kind: LessonKind,
        skill_level: SkillLevel,
        estimated_minutes: u16,
        tags: Vec<String>,
        summary: impl Into<String>,
    ) -> Result<Self, CurriculumError> {
        if estimated_minutes == 0 {
            return Err(CurriculumError::ZeroDuration);
        }
        let title = title.into().trim().to_owned();
        if title.is_empty() {
            return Err(CurriculumError::EmptyTitle);
        }
        let theme = theme.into().trim().to_owned();
        if theme.is_empty() {
            return Err(CurriculumError::EmptyTheme);
        }
        let summary = summary.into().trim().to_owned();
        if summary.is_empty() {
            return Err(CurriculumError::EmptySummary);
        }
        let mut unique_tags = HashSet::new();
        for tag in &tags {
            let normalized = tag.trim();
            if normalized.is_empty() {
                return Err(CurriculumError::EmptyTag);
            }
            if !unique_tags.insert(normalized.to_owned()) {
                return Err(CurriculumError::DuplicateTag(normalized.to_owned()));
            }
        }
        Ok(Self {
            id,
            title,
            theme,
            kind,
            skill_level,
            estimated_minutes,
            tags: unique_tags.into_iter().collect(),
            summary,
        })
    }

    /// Access lesson identifier.
    pub fn id(&self) -> &LessonId {
        &self.id
    }

    /// Access associated theme name.
    pub fn theme(&self) -> &str {
        &self.theme
    }
}

/// A curated list of lessons grouped by theme and level.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Curriculum {
    lessons: Vec<Lesson>,
}

impl Curriculum {
    /// Construct a curriculum and ensure lesson identifiers are unique.
    pub fn new(lessons: Vec<Lesson>) -> Result<Self, CurriculumError> {
        let mut identifiers = HashSet::new();
        for lesson in &lessons {
            if !identifiers.insert(lesson.id.clone()) {
                return Err(CurriculumError::DuplicateLesson(lesson.id.clone()));
            }
        }
        Ok(Self { lessons })
    }

    /// Immutable view of all stored lessons.
    pub fn lessons(&self) -> &[Lesson] {
        &self.lessons
    }

    /// Add a lesson to the curriculum ensuring identifier uniqueness.
    pub fn add_lesson(&mut self, lesson: Lesson) -> Result<(), CurriculumError> {
        if self.lessons.iter().any(|existing| existing.id == lesson.id) {
            return Err(CurriculumError::DuplicateLesson(lesson.id));
        }
        self.lessons.push(lesson);
        Ok(())
    }

    /// Count lessons grouped by their theme.
    pub fn theme_statistics(&self) -> BTreeMap<String, usize> {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for lesson in &self.lessons {
            let entry = counts.entry(lesson.theme.clone()).or_insert(0);
            *entry += 1;
        }
        counts
    }

    /// Retrieve lessons that match the provided theme name.
    pub fn lessons_for_theme(&self, theme: &str) -> Vec<&Lesson> {
        self.lessons
            .iter()
            .filter(|lesson| lesson.theme.eq_ignore_ascii_case(theme))
            .collect()
    }
}

/// Errors that can be encountered while working with the curriculum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CurriculumError {
    EmptyIdentifier,
    EmptyTitle,
    EmptyTheme,
    EmptySummary,
    EmptyTag,
    DuplicateTag(String),
    DuplicateLesson(LessonId),
    ZeroDuration,
}

impl fmt::Display for CurriculumError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentifier => write!(f, "lesson identifier must not be empty"),
            Self::EmptyTitle => write!(f, "lesson title must not be empty"),
            Self::EmptyTheme => write!(f, "lesson theme must not be empty"),
            Self::EmptySummary => write!(f, "lesson summary must not be empty"),
            Self::EmptyTag => write!(f, "lesson tag must not be empty"),
            Self::DuplicateTag(tag) => {
                write!(f, "lesson tag '{tag}' is duplicated within the lesson")
            }
            Self::DuplicateLesson(id) => {
                write!(f, "lesson with identifier '{}' already exists", id.as_str())
            }
            Self::ZeroDuration => write!(f, "lesson duration must be greater than zero minutes"),
        }
    }
}

impl std::error::Error for CurriculumError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_lesson(theme: &str, id_suffix: &str) -> Lesson {
        Lesson::new(
            LessonId::new(format!("lesson-{id_suffix}")).expect("valid id"),
            format!("Урок {id_suffix}"),
            theme,
            LessonKind::Theory,
            SkillLevel::Beginner,
            30,
            vec!["основы".to_string()],
            "Краткое описание",
        )
        .expect("valid lesson")
    }

    #[test]
    fn collects_theme_statistics() {
        let lessons = vec![
            sample_lesson("Rust", "1"),
            sample_lesson("Rust", "2"),
            sample_lesson("Python", "3"),
        ];
        let curriculum = Curriculum::new(lessons).expect("build curriculum");
        let stats = curriculum.theme_statistics();
        assert_eq!(stats.get("Rust"), Some(&2));
        assert_eq!(stats.get("Python"), Some(&1));
    }

    #[test]
    fn prevents_duplicate_lessons() {
        let lesson = sample_lesson("Rust", "dup");
        let result = Curriculum::new(vec![lesson.clone(), lesson]);
        assert!(matches!(result, Err(CurriculumError::DuplicateLesson(_))));
    }

    #[test]
    fn finds_lessons_for_theme_case_insensitive() {
        let lessons = vec![
            sample_lesson("Rust", "1"),
            sample_lesson("python", "2"),
            sample_lesson("Python", "3"),
        ];
        let curriculum = Curriculum::new(lessons).expect("build curriculum");
        let python_lessons = curriculum.lessons_for_theme("PYTHON");
        assert_eq!(python_lessons.len(), 2);
        assert!(python_lessons
            .iter()
            .all(|lesson| lesson.theme().eq_ignore_ascii_case("python")));
    }
}

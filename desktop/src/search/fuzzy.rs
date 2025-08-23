use std::cmp::Ordering;
use std::collections::HashSet;

/// Generate a set of trigrams for the given string
fn trigrams(s: &str) -> HashSet<String> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 3 {
        return HashSet::new();
    }
    chars
        .windows(3)
        .map(|w| w.iter().collect())
        .collect::<HashSet<String>>()
}

/// Calculate trigram similarity between two strings
pub fn similarity(a: &str, b: &str) -> f32 {
    let a = a.to_lowercase();
    let b = b.to_lowercase();
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let ta = trigrams(&a);
    let tb = trigrams(&b);
    if ta.is_empty() || tb.is_empty() {
        return 0.0;
    }
    let inter = ta.intersection(&tb).count() as f32;
    let union = ta.union(&tb).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}

/// Perform fuzzy search over candidates returning those with non-zero score
pub fn search<'a>(query: &str, candidates: impl IntoIterator<Item = &'a str>) -> Vec<(&'a str, f32)> {
    let mut scored: Vec<_> = candidates
        .into_iter()
        .map(|c| (c, similarity(query, c)))
        .filter(|(_, s)| *s > 0.0)
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
    scored
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_best_match_first() {
        let items = vec!["open file", "open folder", "close file"];
        let results = search("open", items.iter().map(|s| *s));
        assert_eq!(results[0].0, "open file");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn similarity_zero_when_no_overlap() {
        assert_eq!(similarity("abc", "xyz"), 0.0);
    }
}

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Generic search index mapping keywords to identifiers.
#[derive(Debug, Default)]
pub struct SearchIndex<ID: Eq + Hash + Clone> {
    map: HashMap<String, Vec<ID>>,
}

impl<ID: Eq + Hash + Clone> SearchIndex<ID> {
    /// Create empty index.
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Insert identifier for given keyword.
    pub fn insert(&mut self, keyword: &str, id: ID) {
        let key = keyword.to_lowercase();
        let ids = self.map.entry(key).or_default();
        if !ids.contains(&id) {
            ids.push(id);
        }
    }

    /// Search index using whitespace separated query.
    /// Returns identifiers matching all query tokens.
    pub fn search(&self, query: &str) -> Vec<ID>
    where
        ID: Ord,
    {
        let tokens: Vec<_> = query.split_whitespace().map(|s| s.to_lowercase()).collect();
        if tokens.is_empty() {
            return Vec::new();
        }
        let mut iter = tokens.into_iter();
        if let Some(first) = iter.next() {
            let mut result: HashSet<ID> = self
                .map
                .get(&first)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect();
            for token in iter {
                if let Some(ids) = self.map.get(&token) {
                    let set: HashSet<ID> = ids.iter().cloned().collect();
                    result = result.intersection(&set).cloned().collect();
                } else {
                    result.clear();
                    break;
                }
            }
            let mut ids: Vec<ID> = result.into_iter().collect();
            ids.sort();
            ids
        } else {
            Vec::new()
        }
    }

    /// Get identifiers for a single keyword.
    pub fn get(&self, keyword: &str) -> Option<&Vec<ID>> {
        self.map.get(&keyword.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_and_searches() {
        let mut idx = SearchIndex::new();
        idx.insert("open", 1);
        idx.insert("file", 1);
        idx.insert("close", 2);
        let res = idx.search("open file");
        assert_eq!(res, vec![1]);
        let res = idx.search("close");
        assert_eq!(res, vec![2]);
        let res = idx.search("missing");
        assert!(res.is_empty());
    }

    #[test]
    fn avoids_duplicate_ids() {
        let mut idx = SearchIndex::new();
        idx.insert("open", 1);
        idx.insert("open", 1);
        let res = idx.search("open");
        assert_eq!(res, vec![1]);
    }

    #[test]
    fn search_with_repeated_keywords_sorted() {
        let mut idx = SearchIndex::new();
        idx.insert("open", 2);
        idx.insert("open", 1);
        idx.insert("open", 2);
        let res = idx.search("open open");
        assert_eq!(res, vec![1, 2]);
    }
}

use std::collections::HashMap;

use crate::domain::value_object::{fold, NotePath};

const EXTENSION: &str = ".md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkResolver {
    by_path: HashMap<String, NotePath>,
    by_name: HashMap<String, Vec<(String, NotePath)>>,
}

impl LinkResolver {
    pub fn new(notes: &[NotePath]) -> Self {
        let mut by_path = HashMap::with_capacity(notes.len());
        let mut by_name: HashMap<String, Vec<(String, NotePath)>> = HashMap::new();
        for note in notes {
            let key = fold(note.without_extension());
            by_path.entry(key.clone()).or_insert_with(|| note.clone());
            by_name
                .entry(last_segment(&key).to_string())
                .or_default()
                .push((key, note.clone()));
        }
        for candidates in by_name.values_mut() {
            candidates.sort_by_key(|(_, note)| {
                (
                    note.as_str().matches('/').count(),
                    note.as_str().to_string(),
                )
            });
        }
        Self { by_path, by_name }
    }

    pub fn resolve(&self, target: &str) -> Option<NotePath> {
        let key = target_key(target);
        if key.is_empty() {
            return None;
        }
        if let Some(exact) = self.by_path.get(&key) {
            return Some(exact.clone());
        }
        let suffix = format!("/{key}");
        self.by_name
            .get(last_segment(&key))?
            .iter()
            .find(|(path, _)| path.ends_with(&suffix))
            .map(|(_, note)| note.clone())
    }
}

fn target_key(target: &str) -> String {
    let folded = fold(target.trim());
    folded
        .strip_suffix(EXTENSION)
        .map_or(folded.clone(), str::to_string)
}

fn last_segment(key: &str) -> &str {
    key.rsplit_once('/').map_or(key, |(_, name)| name)
}

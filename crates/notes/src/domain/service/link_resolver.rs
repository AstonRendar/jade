use crate::domain::value_object::{fold, NotePath};

const EXTENSION: &str = ".md";

#[derive(Debug, Clone, Copy, Default)]
pub struct LinkResolver;

impl LinkResolver {
    pub fn resolve(&self, target: &str, notes: &[NotePath]) -> Option<NotePath> {
        let key = target_key(target);
        if key.is_empty() {
            return None;
        }
        let suffix = format!("/{key}");
        if let Some(exact) = notes.iter().find(|note| path_key(note) == key) {
            return Some(exact.clone());
        }
        notes
            .iter()
            .filter(|note| path_key(note).ends_with(&suffix))
            .min_by_key(|note| {
                (
                    note.as_str().matches('/').count(),
                    note.as_str().to_string(),
                )
            })
            .cloned()
    }
}

fn target_key(target: &str) -> String {
    let folded = fold(target.trim());
    folded
        .strip_suffix(EXTENSION)
        .map_or(folded.clone(), str::to_string)
}

fn path_key(note: &NotePath) -> String {
    fold(note.without_extension())
}

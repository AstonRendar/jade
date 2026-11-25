use std::collections::HashMap;

use crate::domain::value_object::{fold, VaultPath};

const NOTE_EXTENSION: &str = ".md";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileResolver {
    by_path: HashMap<String, VaultPath>,
    by_name: HashMap<String, Vec<(String, VaultPath)>>,
}

impl FileResolver {
    pub fn new(files: &[VaultPath]) -> Self {
        let mut by_path = HashMap::with_capacity(files.len());
        let mut by_name: HashMap<String, Vec<(String, VaultPath)>> = HashMap::new();
        for file in files {
            let key = fold(file.as_str());
            by_path.entry(key.clone()).or_insert_with(|| file.clone());
            by_name
                .entry(fold(file.file_name()))
                .or_default()
                .push((key, file.clone()));
        }
        for candidates in by_name.values_mut() {
            candidates.sort_by_key(|(_, file)| {
                (
                    file.as_str().matches('/').count(),
                    file.as_str().to_string(),
                )
            });
        }
        Self { by_path, by_name }
    }

    pub fn resolve(&self, target: &str) -> Option<VaultPath> {
        let key = fold(
            target
                .trim()
                .trim_start_matches("./")
                .trim_start_matches('/'),
        );
        if key.is_empty() {
            return None;
        }
        self.find(&key)
            .or_else(|| self.find(&format!("{key}{NOTE_EXTENSION}")))
    }

    fn find(&self, key: &str) -> Option<VaultPath> {
        if let Some(exact) = self.by_path.get(key) {
            return Some(exact.clone());
        }
        let name = key.rsplit_once('/').map_or(key, |(_, name)| name);
        let suffix = format!("/{key}");
        self.by_name
            .get(name)?
            .iter()
            .find(|(path, _)| *path == key || path.ends_with(&suffix))
            .map(|(_, file)| file.clone())
    }
}

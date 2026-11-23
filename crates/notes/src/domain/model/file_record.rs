use crate::domain::value_object::{Data, VaultPath};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileMetadata {
    pub size: u64,
    pub created_ms: i64,
    pub modified_ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileRecord {
    path: VaultPath,
    metadata: FileMetadata,
    properties: Vec<(String, Data)>,
    tags: Vec<String>,
    links: Vec<String>,
}

impl FileRecord {
    pub fn new(path: VaultPath, metadata: FileMetadata) -> Self {
        Self {
            path,
            metadata,
            properties: Vec::new(),
            tags: Vec::new(),
            links: Vec::new(),
        }
    }

    pub fn with_contents(
        self,
        properties: Vec<(String, Data)>,
        tags: Vec<String>,
        links: Vec<String>,
    ) -> Self {
        Self {
            properties,
            tags,
            links,
            ..self
        }
    }

    pub fn path(&self) -> &VaultPath {
        &self.path
    }

    pub fn metadata(&self) -> FileMetadata {
        self.metadata
    }

    pub fn properties(&self) -> &[(String, Data)] {
        &self.properties
    }

    pub fn property(&self, name: &str) -> Option<&Data> {
        self.properties
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
    }

    pub fn tags(&self) -> &[String] {
        &self.tags
    }

    pub fn links(&self) -> &[String] {
        &self.links
    }

    pub fn name(&self) -> &str {
        self.path.file_name()
    }

    pub fn extension(&self) -> &str {
        self.name()
            .rsplit_once('.')
            .filter(|(stem, _)| !stem.is_empty())
            .map_or("", |(_, extension)| extension)
    }

    pub fn basename(&self) -> &str {
        let extension = self.extension();
        if extension.is_empty() {
            self.name()
        } else {
            &self.name()[..self.name().len() - extension.len() - 1]
        }
    }

    pub fn folder(&self) -> String {
        self.path
            .parent()
            .map_or_else(String::new, |parent| parent.as_str().to_string())
    }
}

use crate::domain::value_object::NotePath;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Note {
    path: NotePath,
    content: String,
}

impl Note {
    pub fn new(path: NotePath, content: String) -> Self {
        Self { path, content }
    }

    pub fn path(&self) -> &NotePath {
        &self.path
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

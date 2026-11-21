use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkingDirectory(PathBuf);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("working directory `{0}` is not an absolute path")]
pub struct InvalidWorkingDirectoryError(pub String);

impl WorkingDirectory {
    pub fn parse(raw: &str) -> Result<Self, InvalidWorkingDirectoryError> {
        let path = Path::new(raw);
        if path.is_absolute() {
            Ok(Self(path.to_path_buf()))
        } else {
            Err(InvalidWorkingDirectoryError(raw.to_string()))
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

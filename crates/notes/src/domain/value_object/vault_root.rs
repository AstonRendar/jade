use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultRoot(PathBuf);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("vault root `{0}` is not an absolute path")]
pub struct InvalidVaultRootError(pub String);

impl VaultRoot {
    pub fn parse(raw: &str) -> Result<Self, InvalidVaultRootError> {
        let path = Path::new(raw);
        if path.is_absolute() {
            Ok(Self(path.to_path_buf()))
        } else {
            Err(InvalidVaultRootError(raw.to_string()))
        }
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }

    pub fn name(&self) -> String {
        self.0.file_name().map_or_else(
            || self.0.display().to_string(),
            |name| name.to_string_lossy().into_owned(),
        )
    }
}

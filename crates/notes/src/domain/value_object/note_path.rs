use super::VaultPath;

const EXTENSION: &str = ".md";

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NotePath(VaultPath);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid note path `{0}`")]
pub struct InvalidNotePathError(pub String);

impl NotePath {
    pub fn parse(raw: &str) -> Result<Self, InvalidNotePathError> {
        let path = VaultPath::parse(raw).map_err(|_| InvalidNotePathError(raw.to_string()))?;
        Self::from_vault_path(path)
    }

    pub fn from_vault_path(path: VaultPath) -> Result<Self, InvalidNotePathError> {
        if has_markdown_name(path.file_name()) {
            Ok(Self(path))
        } else {
            Err(InvalidNotePathError(path.as_str().to_string()))
        }
    }

    pub fn from_title(
        folder: Option<&VaultPath>,
        title: &str,
    ) -> Result<Self, InvalidNotePathError> {
        let title = title.trim();
        if title.is_empty() || title == "." || title == ".." || title.contains(['/', '\\']) {
            return Err(InvalidNotePathError(title.to_string()));
        }
        let file_name = format!("{title}{EXTENSION}");
        let path = match folder {
            Some(folder) => folder.join(&file_name),
            None => VaultPath::parse(&file_name),
        }
        .map_err(|_| InvalidNotePathError(title.to_string()))?;
        Self::from_vault_path(path)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_vault_path(&self) -> &VaultPath {
        &self.0
    }

    pub fn title(&self) -> &str {
        let name = self.0.file_name();
        &name[..name.len() - EXTENSION.len()]
    }

    pub(crate) fn without_extension(&self) -> &str {
        let path = self.0.as_str();
        &path[..path.len() - EXTENSION.len()]
    }
}

fn has_markdown_name(file_name: &str) -> bool {
    file_name.len() > EXTENSION.len() && file_name.to_lowercase().ends_with(EXTENSION)
}

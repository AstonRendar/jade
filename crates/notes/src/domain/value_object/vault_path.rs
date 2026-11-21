use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct VaultPath(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid vault path `{0}`")]
pub struct InvalidVaultPathError(pub String);

impl VaultPath {
    pub fn parse(raw: &str) -> Result<Self, InvalidVaultPathError> {
        let normalised: String = raw.nfc().collect();
        if is_valid(&normalised) {
            Ok(Self(normalised))
        } else {
            Err(InvalidVaultPathError(raw.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn file_name(&self) -> &str {
        self.0.rsplit_once('/').map_or(&self.0, |(_, name)| name)
    }

    pub fn parent(&self) -> Option<VaultPath> {
        self.0
            .rsplit_once('/')
            .map(|(parent, _)| Self(parent.to_string()))
    }

    pub fn is_hidden(&self) -> bool {
        self.0.split('/').any(|segment| segment.starts_with('.'))
    }

    pub fn join(&self, name: &str) -> Result<VaultPath, InvalidVaultPathError> {
        if name.contains('/') {
            return Err(InvalidVaultPathError(name.to_string()));
        }
        Self::parse(&format!("{}/{name}", self.0))
    }

    pub fn is_within(&self, folder: &VaultPath) -> bool {
        self == folder || self.0.starts_with(&format!("{}/", folder.0))
    }
}

fn is_valid(path: &str) -> bool {
    !path.contains('\\')
        && !path.chars().any(char::is_control)
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

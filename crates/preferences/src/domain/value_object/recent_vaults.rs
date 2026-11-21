use std::path::Path;

const MAX_RECENT_VAULTS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecentVaults(Vec<String>);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("vault path `{0}` is not absolute")]
pub struct RelativeVaultPathError(pub String);

impl RecentVaults {
    pub fn remember(&self, path: &str) -> Result<Self, RelativeVaultPathError> {
        if !Path::new(path).is_absolute() {
            return Err(RelativeVaultPathError(path.to_string()));
        }
        let mut paths = vec![path.to_string()];
        paths.extend(self.0.iter().filter(|known| *known != path).cloned());
        paths.truncate(MAX_RECENT_VAULTS);
        Ok(Self(paths))
    }

    pub fn forget(&self, path: &str) -> Self {
        Self(
            self.0
                .iter()
                .filter(|known| *known != path)
                .cloned()
                .collect(),
        )
    }

    pub fn paths(&self) -> &[String] {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TrackedPath(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid tracked path `{0}`")]
pub struct InvalidTrackedPathError(pub String);

impl TrackedPath {
    pub fn parse(raw: &str) -> Result<Self, InvalidTrackedPathError> {
        if is_valid(raw) {
            Ok(Self(raw.to_string()))
        } else {
            Err(InvalidTrackedPathError(raw.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn folder(&self) -> Option<&str> {
        self.0.rsplit_once('/').map(|(folder, _)| folder)
    }

    pub fn file_name(&self) -> &str {
        self.0.rsplit_once('/').map_or(&self.0, |(_, name)| name)
    }

    pub(crate) fn sibling(&self, file_name: &str) -> Self {
        match self.folder() {
            Some(folder) => Self(format!("{folder}/{file_name}")),
            None => Self(file_name.to_string()),
        }
    }
}

fn is_valid(path: &str) -> bool {
    !path.contains('\\')
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

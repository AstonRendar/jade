const FORBIDDEN_CHARACTERS: [char; 8] = [' ', '~', '^', ':', '?', '*', '[', '\\'];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BranchName(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid branch name `{0}`")]
pub struct InvalidBranchNameError(pub String);

impl BranchName {
    pub fn parse(raw: &str) -> Result<Self, InvalidBranchNameError> {
        if is_valid(raw) {
            Ok(Self(raw.to_string()))
        } else {
            Err(InvalidBranchNameError(raw.to_string()))
        }
    }

    pub fn main() -> Self {
        Self("main".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_valid(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with(['-', '/'])
        && !name.ends_with(['/', '.'])
        && !name.ends_with(".lock")
        && !name.contains("..")
        && !name.contains("//")
        && !name.contains("@{")
        && !name
            .chars()
            .any(|c| c.is_control() || FORBIDDEN_CHARACTERS.contains(&c))
}

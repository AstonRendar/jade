const SHA1_LENGTH: usize = 40;
const SHA256_LENGTH: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CommitId(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("invalid commit id `{0}`")]
pub struct InvalidCommitIdError(pub String);

impl CommitId {
    pub fn parse(raw: &str) -> Result<Self, InvalidCommitIdError> {
        let has_valid_length = raw.len() == SHA1_LENGTH || raw.len() == SHA256_LENGTH;
        if has_valid_length && raw.chars().all(|c| c.is_ascii_hexdigit()) {
            Ok(Self(raw.to_ascii_lowercase()))
        } else {
            Err(InvalidCommitIdError(raw.to_string()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

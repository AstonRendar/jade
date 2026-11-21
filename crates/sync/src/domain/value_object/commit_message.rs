use super::DeviceName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitMessage(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("commit message is empty")]
pub struct EmptyCommitMessageError;

impl CommitMessage {
    pub fn new(text: &str) -> Result<Self, EmptyCommitMessageError> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(EmptyCommitMessageError);
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn sync_from(device: &DeviceName) -> Self {
        Self(format!("Sync from {}", device.as_str()))
    }

    pub fn merge_remote_changes() -> Self {
        Self("Merge remote changes".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

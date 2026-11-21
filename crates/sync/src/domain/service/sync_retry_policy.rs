const DEFAULT_MAX_ATTEMPTS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncRetryPolicy {
    max_attempts: u32,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("a retry policy needs at least one attempt")]
pub struct InvalidRetryPolicyError;

impl SyncRetryPolicy {
    pub fn with_max_attempts(max_attempts: u32) -> Result<Self, InvalidRetryPolicyError> {
        if max_attempts == 0 {
            return Err(InvalidRetryPolicyError);
        }
        Ok(Self { max_attempts })
    }

    pub fn allows_another_attempt(&self, attempts_made: u32) -> bool {
        attempts_made < self.max_attempts
    }
}

impl Default for SyncRetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: DEFAULT_MAX_ATTEMPTS,
        }
    }
}

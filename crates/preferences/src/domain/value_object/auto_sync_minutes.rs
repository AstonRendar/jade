const DEFAULT_MINUTES: u32 = 5;
const MAX_MINUTES: u32 = 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AutoSyncMinutes(u32);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("automatic sync every {0} minutes is longer than a day")]
pub struct AutoSyncTooLongError(pub u32);

impl AutoSyncMinutes {
    pub fn new(minutes: u32) -> Result<Self, AutoSyncTooLongError> {
        if minutes > MAX_MINUTES {
            return Err(AutoSyncTooLongError(minutes));
        }
        Ok(Self(minutes))
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl Default for AutoSyncMinutes {
    fn default() -> Self {
        Self(DEFAULT_MINUTES)
    }
}

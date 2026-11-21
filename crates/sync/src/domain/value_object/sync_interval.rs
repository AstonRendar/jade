const DEFAULT_MINUTES: u32 = 5;
const MAX_MINUTES: u32 = 24 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncInterval {
    minutes: u32,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("sync interval of {0} minutes is longer than a day")]
pub struct SyncIntervalTooLongError(pub u32);

impl SyncInterval {
    pub fn from_minutes(minutes: u32) -> Result<Self, SyncIntervalTooLongError> {
        if minutes > MAX_MINUTES {
            return Err(SyncIntervalTooLongError(minutes));
        }
        Ok(Self { minutes })
    }

    pub fn minutes(&self) -> u32 {
        self.minutes
    }

    pub fn is_enabled(&self) -> bool {
        self.minutes > 0
    }
}

impl Default for SyncInterval {
    fn default() -> Self {
        Self {
            minutes: DEFAULT_MINUTES,
        }
    }
}

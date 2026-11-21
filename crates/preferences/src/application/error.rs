use crate::domain::port::PreferencesStorageError;
use crate::domain::value_object::{
    AutoSyncTooLongError, RelativeVaultPathError, UnknownThemeError, UnsupportedLocaleError,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PreferencesError {
    #[error("unsupported locale `{0}`")]
    InvalidLocale(String),
    #[error("unknown theme `{0}`")]
    InvalidTheme(String),
    #[error("automatic sync every {0} minutes is not allowed")]
    InvalidAutoSync(u32),
    #[error("vault path `{0}` is not absolute")]
    InvalidVaultPath(String),
    #[error("preferences storage failure: {0}")]
    Storage(String),
}

impl From<UnsupportedLocaleError> for PreferencesError {
    fn from(error: UnsupportedLocaleError) -> Self {
        Self::InvalidLocale(error.0)
    }
}

impl From<UnknownThemeError> for PreferencesError {
    fn from(error: UnknownThemeError) -> Self {
        Self::InvalidTheme(error.0)
    }
}

impl From<AutoSyncTooLongError> for PreferencesError {
    fn from(error: AutoSyncTooLongError) -> Self {
        Self::InvalidAutoSync(error.0)
    }
}

impl From<RelativeVaultPathError> for PreferencesError {
    fn from(error: RelativeVaultPathError) -> Self {
        Self::InvalidVaultPath(error.0)
    }
}

impl From<PreferencesStorageError> for PreferencesError {
    fn from(error: PreferencesStorageError) -> Self {
        Self::Storage(error.0)
    }
}

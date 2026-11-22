use crate::domain::model::Preferences;

pub trait PreferencesRepository {
    fn load(&self) -> Result<Preferences, PreferencesStorageError>;
    fn save(&self, preferences: &Preferences) -> Result<(), PreferencesStorageError>;
}

impl<R: PreferencesRepository> PreferencesRepository for &R {
    fn load(&self) -> Result<Preferences, PreferencesStorageError> {
        (**self).load()
    }

    fn save(&self, preferences: &Preferences) -> Result<(), PreferencesStorageError> {
        (**self).save(preferences)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("preferences storage failure: {0}")]
pub struct PreferencesStorageError(pub String);

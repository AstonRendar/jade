use jade_preferences::domain::model::Preferences;
use jade_preferences::domain::port::{PreferencesRepository, PreferencesStorageError};
use mockall::mock;

mock! {
    pub PreferencesRepository {}

    impl PreferencesRepository for PreferencesRepository {
        fn load(&self) -> Result<Preferences, PreferencesStorageError>;
        fn save(&self, preferences: &Preferences) -> Result<(), PreferencesStorageError>;
    }
}

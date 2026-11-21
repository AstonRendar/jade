use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::model::Preferences;
use crate::domain::port::{PreferencesRepository, PreferencesStorageError};
use crate::domain::value_object::{AutoSyncMinutes, Locale, RecentVaults, Theme};

#[derive(Debug, Clone)]
pub struct JsonFilePreferencesRepository {
    file: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct PreferencesDocument {
    locale: Option<String>,
    theme: String,
    auto_sync_minutes: u32,
    recent_vaults: Vec<String>,
}

impl JsonFilePreferencesRepository {
    pub fn new(file: PathBuf) -> Self {
        Self { file }
    }
}

impl PreferencesRepository for JsonFilePreferencesRepository {
    fn load(&self) -> Result<Preferences, PreferencesStorageError> {
        if !self.file.exists() {
            return Ok(Preferences::default());
        }
        let text = fs::read_to_string(&self.file).map_err(storage)?;
        let document: PreferencesDocument = serde_json::from_str(&text).map_err(storage)?;
        to_preferences(document)
    }

    fn save(&self, preferences: &Preferences) -> Result<(), PreferencesStorageError> {
        let text = serde_json::to_string_pretty(&to_document(preferences)).map_err(storage)?;
        if let Some(folder) = self.file.parent() {
            fs::create_dir_all(folder).map_err(storage)?;
        }
        let temporary = self.file.with_extension("json.tmp");
        fs::write(&temporary, text).map_err(storage)?;
        fs::rename(&temporary, &self.file).map_err(storage)
    }
}

fn to_document(preferences: &Preferences) -> PreferencesDocument {
    PreferencesDocument {
        locale: preferences
            .locale()
            .map(|locale| locale.as_str().to_string()),
        theme: preferences.theme().as_str().to_string(),
        auto_sync_minutes: preferences.auto_sync_minutes().value(),
        recent_vaults: preferences.recent_vaults().paths().to_vec(),
    }
}

fn to_preferences(document: PreferencesDocument) -> Result<Preferences, PreferencesStorageError> {
    let locale = document
        .locale
        .as_deref()
        .map(Locale::parse)
        .transpose()
        .map_err(storage)?;
    let theme = Theme::parse(&document.theme).map_err(storage)?;
    let auto_sync_minutes = AutoSyncMinutes::new(document.auto_sync_minutes).map_err(storage)?;
    let mut recent_vaults = RecentVaults::default();
    for path in document.recent_vaults.iter().rev() {
        recent_vaults = recent_vaults.remember(path).map_err(storage)?;
    }
    Ok(Preferences::restore(
        locale,
        theme,
        auto_sync_minutes,
        recent_vaults,
    ))
}

fn storage(error: impl std::error::Error) -> PreferencesStorageError {
    PreferencesStorageError(error.to_string())
}

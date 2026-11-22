use jade_shared::application::use_case::UseCase;

use super::error::PreferencesError;
use crate::domain::model::Preferences;
use crate::domain::port::PreferencesRepository;
use crate::domain::value_object::{AutoSyncMinutes, Locale, Theme};

pub struct UpdatePreferencesRequest {
    pub locale: Option<String>,
    pub theme: String,
    pub auto_sync_minutes: u32,
}

pub struct UpdatePreferences<R> {
    repository: R,
}

impl<R: PreferencesRepository> UpdatePreferences<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PreferencesRepository> UseCase for UpdatePreferences<R> {
    type Request = UpdatePreferencesRequest;
    type Response = Preferences;
    type Error = PreferencesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let locale = request.locale.as_deref().map(Locale::parse).transpose()?;
        let theme = Theme::parse(&request.theme)?;
        let auto_sync_minutes = AutoSyncMinutes::new(request.auto_sync_minutes)?;
        let preferences = self
            .repository
            .load()?
            .with_settings(locale, theme, auto_sync_minutes);
        self.repository.save(&preferences)?;
        Ok(preferences)
    }
}

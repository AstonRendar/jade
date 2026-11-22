use jade_shared::application::use_case::UseCase;

use super::error::PreferencesError;
use crate::domain::model::Preferences;
use crate::domain::port::PreferencesRepository;

pub struct RememberVaultRequest {
    pub path: String,
}

pub struct RememberVault<R> {
    repository: R,
}

impl<R: PreferencesRepository> RememberVault<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PreferencesRepository> UseCase for RememberVault<R> {
    type Request = RememberVaultRequest;
    type Response = Preferences;
    type Error = PreferencesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let preferences = self.repository.load()?.remember_vault(&request.path)?;
        self.repository.save(&preferences)?;
        Ok(preferences)
    }
}

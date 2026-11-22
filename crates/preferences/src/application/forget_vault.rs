use jade_shared::application::use_case::UseCase;

use super::error::PreferencesError;
use crate::domain::model::Preferences;
use crate::domain::port::PreferencesRepository;

pub struct ForgetVaultRequest {
    pub path: String,
}

pub struct ForgetVault<R> {
    repository: R,
}

impl<R: PreferencesRepository> ForgetVault<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PreferencesRepository> UseCase for ForgetVault<R> {
    type Request = ForgetVaultRequest;
    type Response = Preferences;
    type Error = PreferencesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let preferences = self.repository.load()?.forget_vault(&request.path);
        self.repository.save(&preferences)?;
        Ok(preferences)
    }
}

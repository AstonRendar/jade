use jade_shared::application::use_case::UseCase;

use super::error::PreferencesError;
use crate::domain::model::Preferences;
use crate::domain::port::PreferencesRepository;

pub struct GetPreferences<R> {
    repository: R,
}

impl<R: PreferencesRepository> GetPreferences<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: PreferencesRepository> UseCase for GetPreferences<R> {
    type Request = ();
    type Response = Preferences;
    type Error = PreferencesError;

    fn execute(&self, _: Self::Request) -> Result<Self::Response, Self::Error> {
        Ok(self.repository.load()?)
    }
}

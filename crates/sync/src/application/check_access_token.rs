use jade_shared::application::use_case::UseCase;

use crate::domain::port::{CredentialStore, CredentialStoreError};

pub struct CheckAccessToken<C> {
    credentials: C,
}

impl<C: CredentialStore> CheckAccessToken<C> {
    pub fn new(credentials: C) -> Self {
        Self { credentials }
    }
}

impl<C: CredentialStore> UseCase for CheckAccessToken<C> {
    type Request = ();
    type Response = bool;
    type Error = CredentialStoreError;

    fn execute(&self, _: Self::Request) -> Result<Self::Response, Self::Error> {
        Ok(self.credentials.load()?.is_some())
    }
}

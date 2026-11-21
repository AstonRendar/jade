use std::sync::Mutex;

use crate::domain::port::{CredentialStore, CredentialStoreError};
use crate::domain::value_object::AccessToken;

#[derive(Debug, Default)]
pub struct InMemoryCredentialStore {
    token: Mutex<Option<AccessToken>>,
}

impl InMemoryCredentialStore {
    fn with_token<T>(
        &self,
        action: impl FnOnce(&mut Option<AccessToken>) -> T,
    ) -> Result<T, CredentialStoreError> {
        let mut token = self
            .token
            .lock()
            .map_err(|error| CredentialStoreError(error.to_string()))?;
        Ok(action(&mut token))
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn load(&self) -> Result<Option<AccessToken>, CredentialStoreError> {
        self.with_token(|token| token.clone())
    }

    fn save(&self, token: &AccessToken) -> Result<(), CredentialStoreError> {
        self.with_token(|stored| *stored = Some(token.clone()))
    }

    fn delete(&self) -> Result<(), CredentialStoreError> {
        self.with_token(|stored| *stored = None)
    }
}

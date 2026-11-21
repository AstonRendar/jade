use keyring::{Entry, Error};

use crate::domain::port::{CredentialStore, CredentialStoreError};
use crate::domain::value_object::AccessToken;

const ACCOUNT: &str = "github-token";

#[derive(Debug, Clone)]
pub struct KeyringCredentialStore {
    service: String,
}

impl KeyringCredentialStore {
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_string(),
        }
    }

    fn entry(&self) -> Result<Entry, CredentialStoreError> {
        Entry::new(&self.service, ACCOUNT).map_err(unavailable)
    }
}

impl CredentialStore for KeyringCredentialStore {
    fn load(&self) -> Result<Option<AccessToken>, CredentialStoreError> {
        match self.entry()?.get_password() {
            Ok(secret) => AccessToken::new(&secret).map(Some).map_err(unavailable),
            Err(Error::NoEntry) => Ok(None),
            Err(error) => Err(unavailable(error)),
        }
    }

    fn save(&self, token: &AccessToken) -> Result<(), CredentialStoreError> {
        self.entry()?
            .set_password(token.expose())
            .map_err(unavailable)
    }

    fn delete(&self) -> Result<(), CredentialStoreError> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(Error::NoEntry) => Ok(()),
            Err(error) => Err(unavailable(error)),
        }
    }
}

fn unavailable(error: impl std::error::Error) -> CredentialStoreError {
    CredentialStoreError(error.to_string())
}

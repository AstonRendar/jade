use std::sync::OnceLock;

use keyring_core::{Entry, Error};

use crate::domain::port::{CredentialStore, CredentialStoreError};
use crate::domain::value_object::AccessToken;

const ACCOUNT: &str = "github-token";

static PLATFORM_STORE: OnceLock<Result<(), String>> = OnceLock::new();

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
        PLATFORM_STORE
            .get_or_init(|| use_platform_store().map_err(|error| error.to_string()))
            .clone()
            .map_err(CredentialStoreError)?;
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

#[cfg(any(target_os = "macos", target_os = "ios"))]
fn use_platform_store() -> keyring_core::Result<()> {
    keyring_core::set_default_store(apple_native_keyring_store::keychain::Store::new()?);
    Ok(())
}

#[cfg(target_os = "android")]
fn use_platform_store() -> keyring_core::Result<()> {
    keyring_core::set_default_store(android_native_keyring_store::Store::new()?);
    Ok(())
}

#[cfg(target_os = "windows")]
fn use_platform_store() -> keyring_core::Result<()> {
    keyring_core::set_default_store(windows_native_keyring_store::Store::new()?);
    Ok(())
}

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "ios", target_os = "android"))
))]
fn use_platform_store() -> keyring_core::Result<()> {
    keyring_core::set_default_store(zbus_secret_service_keyring_store::Store::new()?);
    Ok(())
}

fn unavailable(error: impl std::error::Error) -> CredentialStoreError {
    CredentialStoreError(error.to_string())
}

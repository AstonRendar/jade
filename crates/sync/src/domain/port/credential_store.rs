use crate::domain::value_object::AccessToken;

pub trait CredentialStore {
    fn load(&self) -> Result<Option<AccessToken>, CredentialStoreError>;
    fn save(&self, token: &AccessToken) -> Result<(), CredentialStoreError>;
    fn delete(&self) -> Result<(), CredentialStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("credential store unavailable: {0}")]
pub struct CredentialStoreError(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Credentials {
    Anonymous,
    Token(AccessToken),
}

impl From<Option<AccessToken>> for Credentials {
    fn from(token: Option<AccessToken>) -> Self {
        token.map_or(Self::Anonymous, Self::Token)
    }
}

impl<C: CredentialStore> CredentialStore for &C {
    fn load(&self) -> Result<Option<AccessToken>, CredentialStoreError> {
        (**self).load()
    }

    fn save(&self, token: &AccessToken) -> Result<(), CredentialStoreError> {
        (**self).save(token)
    }

    fn delete(&self) -> Result<(), CredentialStoreError> {
        (**self).delete()
    }
}

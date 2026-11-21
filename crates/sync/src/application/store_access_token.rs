use jade_shared::application::use_case::UseCase;

use crate::domain::port::{CredentialStore, CredentialStoreError};
use crate::domain::value_object::{AccessToken, InvalidAccessTokenError};

pub struct StoreAccessTokenRequest {
    pub token: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum StoreAccessTokenError {
    #[error(transparent)]
    InvalidToken(#[from] InvalidAccessTokenError),
    #[error(transparent)]
    Credentials(#[from] CredentialStoreError),
}

pub struct StoreAccessToken<C> {
    credentials: C,
}

impl<C: CredentialStore> StoreAccessToken<C> {
    pub fn new(credentials: C) -> Self {
        Self { credentials }
    }
}

impl<C: CredentialStore> UseCase for StoreAccessToken<C> {
    type Request = StoreAccessTokenRequest;
    type Response = ();
    type Error = StoreAccessTokenError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        if request.token.is_empty() {
            return Ok(self.credentials.delete()?);
        }
        let token = AccessToken::new(&request.token)?;
        Ok(self.credentials.save(&token)?)
    }
}

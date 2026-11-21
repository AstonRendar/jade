use std::cell::Cell;

use git2::{Cred, CredentialType, ErrorClass, ErrorCode, RemoteCallbacks};

use crate::domain::port::{Credentials, VersionControlError};

const MAX_ATTEMPTS: u32 = 2;
const TOKEN_USERNAME: &str = "x-access-token";

pub struct Git2Authentication<'a> {
    credentials: &'a Credentials,
    attempts: Cell<u32>,
    anonymous_rejected: Cell<bool>,
}

impl<'a> Git2Authentication<'a> {
    pub fn new(credentials: &'a Credentials) -> Self {
        Self {
            credentials,
            attempts: Cell::new(0),
            anonymous_rejected: Cell::new(false),
        }
    }

    pub fn callbacks(&self) -> RemoteCallbacks<'_> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_, _, allowed| self.provide(allowed));
        callbacks
    }

    pub fn error(&self, error: git2::Error) -> VersionControlError {
        if self.anonymous_rejected.get() {
            VersionControlError::AuthenticationRequired
        } else if self.attempts.get() > 0 || error.code() == ErrorCode::Auth {
            VersionControlError::AuthenticationFailed
        } else {
            network_or_backend(error)
        }
    }

    fn provide(&self, allowed: CredentialType) -> Result<Cred, git2::Error> {
        self.attempts.set(self.attempts.get() + 1);
        match self.credentials {
            Credentials::Token(token)
                if self.attempts.get() <= MAX_ATTEMPTS
                    && allowed.contains(CredentialType::USER_PASS_PLAINTEXT) =>
            {
                Cred::userpass_plaintext(TOKEN_USERNAME, token.expose())
            }
            Credentials::Token(_) => Err(git2::Error::from_str("authentication failed")),
            Credentials::Anonymous => {
                self.anonymous_rejected.set(true);
                Err(git2::Error::from_str("authentication required"))
            }
        }
    }
}

pub fn network_or_backend(error: git2::Error) -> VersionControlError {
    match error.class() {
        ErrorClass::Net | ErrorClass::Http | ErrorClass::Ssl | ErrorClass::Ssh => {
            VersionControlError::Network(error.message().to_string())
        }
        _ => VersionControlError::Backend(error.message().to_string()),
    }
}

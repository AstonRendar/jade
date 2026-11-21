use jade_shared::application::use_case::UseCase;

use crate::domain::port::{
    CredentialStore, CredentialStoreError, Credentials, VersionControl, VersionControlError,
};
use crate::domain::value_object::{
    InvalidWorkingDirectoryError, RemoteUrl, UnsupportedRemoteUrlError, WorkingDirectory,
};

pub struct CloneRepositoryRequest {
    pub url: String,
    pub working_directory: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CloneRepositoryError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    InvalidRemoteUrl(#[from] UnsupportedRemoteUrlError),
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
    #[error(transparent)]
    Credentials(#[from] CredentialStoreError),
}

pub struct CloneRepository<V, C> {
    version_control: V,
    credentials: C,
}

impl<V: VersionControl, C: CredentialStore> CloneRepository<V, C> {
    pub fn new(version_control: V, credentials: C) -> Self {
        Self {
            version_control,
            credentials,
        }
    }
}

impl<V: VersionControl, C: CredentialStore> UseCase for CloneRepository<V, C> {
    type Request = CloneRepositoryRequest;
    type Response = ();
    type Error = CloneRepositoryError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let url = RemoteUrl::parse(&request.url)?;
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        let credentials = Credentials::from(self.credentials.load()?);
        self.version_control.clone_from(&url, &dir, &credentials)?;
        Ok(())
    }
}

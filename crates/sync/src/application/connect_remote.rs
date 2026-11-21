use jade_shared::application::use_case::UseCase;

use crate::domain::port::{VersionControl, VersionControlError};
use crate::domain::value_object::{
    InvalidWorkingDirectoryError, RemoteUrl, UnsupportedRemoteUrlError, WorkingDirectory,
};

pub struct ConnectRemoteRequest {
    pub working_directory: String,
    pub url: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ConnectRemoteError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    InvalidRemoteUrl(#[from] UnsupportedRemoteUrlError),
    #[error("the folder is not a repository")]
    NotARepository,
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
}

pub struct ConnectRemote<V> {
    version_control: V,
}

impl<V: VersionControl> ConnectRemote<V> {
    pub fn new(version_control: V) -> Self {
        Self { version_control }
    }
}

impl<V: VersionControl> UseCase for ConnectRemote<V> {
    type Request = ConnectRemoteRequest;
    type Response = ();
    type Error = ConnectRemoteError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        let url = RemoteUrl::parse(&request.url)?;
        if !self.version_control.is_repository(&dir)? {
            return Err(ConnectRemoteError::NotARepository);
        }
        self.version_control.set_remote(&dir, &url)?;
        Ok(())
    }
}

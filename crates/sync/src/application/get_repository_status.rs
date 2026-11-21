use jade_shared::application::use_case::UseCase;

use crate::domain::port::{RepositoryStatus, VersionControl, VersionControlError};
use crate::domain::value_object::{InvalidWorkingDirectoryError, WorkingDirectory};

pub struct GetRepositoryStatusRequest {
    pub working_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryState {
    NotARepository,
    Ready(RepositoryStatus),
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetRepositoryStatusError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
}

pub struct GetRepositoryStatus<V> {
    version_control: V,
}

impl<V: VersionControl> GetRepositoryStatus<V> {
    pub fn new(version_control: V) -> Self {
        Self { version_control }
    }
}

impl<V: VersionControl> UseCase for GetRepositoryStatus<V> {
    type Request = GetRepositoryStatusRequest;
    type Response = RepositoryState;
    type Error = GetRepositoryStatusError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        if !self.version_control.is_repository(&dir)? {
            return Ok(RepositoryState::NotARepository);
        }
        Ok(RepositoryState::Ready(self.version_control.status(&dir)?))
    }
}

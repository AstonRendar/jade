use jade_shared::application::use_case::UseCase;

use crate::domain::port::{VersionControl, VersionControlError};
use crate::domain::value_object::{BranchName, InvalidWorkingDirectoryError, WorkingDirectory};

pub struct InitRepositoryRequest {
    pub working_directory: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InitRepositoryError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error("the folder is already a repository")]
    AlreadyARepository,
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
}

pub struct InitRepository<V> {
    version_control: V,
}

impl<V: VersionControl> InitRepository<V> {
    pub fn new(version_control: V) -> Self {
        Self { version_control }
    }
}

impl<V: VersionControl> UseCase for InitRepository<V> {
    type Request = InitRepositoryRequest;
    type Response = ();
    type Error = InitRepositoryError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        if self.version_control.is_repository(&dir)? {
            return Err(InitRepositoryError::AlreadyARepository);
        }
        self.version_control.init(&dir, &BranchName::main())?;
        Ok(())
    }
}

use jade_shared::application::use_case::UseCase;

use crate::domain::port::{VersionControl, VersionControlError};
use crate::domain::value_object::{
    CommitId, InvalidCommitIdError, InvalidTrackedPathError, InvalidWorkingDirectoryError,
    TrackedPath, WorkingDirectory,
};

pub struct ReadNoteRevisionRequest {
    pub working_directory: String,
    pub path: String,
    pub commit: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ReadNoteRevisionError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    InvalidPath(#[from] InvalidTrackedPathError),
    #[error(transparent)]
    InvalidCommitId(#[from] InvalidCommitIdError),
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
}

pub struct ReadNoteRevision<V> {
    version_control: V,
}

impl<V: VersionControl> ReadNoteRevision<V> {
    pub fn new(version_control: V) -> Self {
        Self { version_control }
    }
}

impl<V: VersionControl> UseCase for ReadNoteRevision<V> {
    type Request = ReadNoteRevisionRequest;
    type Response = Vec<u8>;
    type Error = ReadNoteRevisionError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        let path = TrackedPath::parse(&request.path)?;
        let commit = CommitId::parse(&request.commit)?;
        Ok(self.version_control.read_at(&dir, &path, &commit)?)
    }
}

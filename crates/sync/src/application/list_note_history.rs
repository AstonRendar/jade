use jade_shared::application::use_case::UseCase;

use crate::domain::port::{Revision, VersionControl, VersionControlError};
use crate::domain::value_object::{
    InvalidTrackedPathError, InvalidWorkingDirectoryError, TrackedPath, WorkingDirectory,
};

pub struct ListNoteHistoryRequest {
    pub working_directory: String,
    pub path: String,
    pub limit: usize,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListNoteHistoryError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    InvalidPath(#[from] InvalidTrackedPathError),
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
}

pub struct ListNoteHistory<V> {
    version_control: V,
}

impl<V: VersionControl> ListNoteHistory<V> {
    pub fn new(version_control: V) -> Self {
        Self { version_control }
    }
}

impl<V: VersionControl> UseCase for ListNoteHistory<V> {
    type Request = ListNoteHistoryRequest;
    type Response = Vec<Revision>;
    type Error = ListNoteHistoryError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        let path = TrackedPath::parse(&request.path)?;
        Ok(self.version_control.history(&dir, &path, request.limit)?)
    }
}

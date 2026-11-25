use jade_shared::application::use_case::UseCase;

use super::list_branches::{refresh_remote_branches, ListBranchesError};
use crate::domain::port::{
    CredentialStore, CredentialStoreError, DeviceInfo, VersionControl, VersionControlError,
};
use crate::domain::value_object::{
    BranchName, CommitMessage, InvalidBranchNameError, InvalidWorkingDirectoryError,
    WorkingDirectory,
};

pub struct SwitchBranchRequest {
    pub working_directory: String,
    pub branch: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SwitchBranchError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    InvalidBranch(#[from] InvalidBranchNameError),
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
    #[error(transparent)]
    Credentials(#[from] CredentialStoreError),
}

impl From<ListBranchesError> for SwitchBranchError {
    fn from(error: ListBranchesError) -> Self {
        match error {
            ListBranchesError::InvalidWorkingDirectory(error) => error.into(),
            ListBranchesError::VersionControl(error) => error.into(),
            ListBranchesError::Credentials(error) => error.into(),
        }
    }
}

pub struct SwitchBranch<V, C, D> {
    version_control: V,
    credentials: C,
    device: D,
}

impl<V: VersionControl, C: CredentialStore, D: DeviceInfo> SwitchBranch<V, C, D> {
    pub fn new(version_control: V, credentials: C, device: D) -> Self {
        Self {
            version_control,
            credentials,
            device,
        }
    }
}

impl<V: VersionControl, C: CredentialStore, D: DeviceInfo> UseCase for SwitchBranch<V, C, D> {
    type Request = SwitchBranchRequest;
    type Response = ();
    type Error = SwitchBranchError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        let branch = BranchName::parse(&request.branch)?;
        if self.version_control.current_branch(&dir)? == branch {
            return Ok(());
        }
        let message = CommitMessage::sync_from(&self.device.device_name());
        self.version_control.commit_all(&dir, &message)?;
        refresh_remote_branches(&self.version_control, &self.credentials, &dir)?;
        Ok(self.version_control.switch_branch(&dir, &branch)?)
    }
}

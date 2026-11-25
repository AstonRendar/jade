use jade_shared::application::use_case::UseCase;

use crate::domain::port::{
    CredentialStore, CredentialStoreError, Credentials, VersionControl, VersionControlError,
};
use crate::domain::value_object::{BranchName, InvalidWorkingDirectoryError, WorkingDirectory};

pub struct ListBranchesRequest {
    pub working_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchList {
    pub current: BranchName,
    pub branches: Vec<BranchName>,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ListBranchesError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
    #[error(transparent)]
    Credentials(#[from] CredentialStoreError),
}

pub struct ListBranches<V, C> {
    version_control: V,
    credentials: C,
}

impl<V: VersionControl, C: CredentialStore> ListBranches<V, C> {
    pub fn new(version_control: V, credentials: C) -> Self {
        Self {
            version_control,
            credentials,
        }
    }
}

impl<V: VersionControl, C: CredentialStore> UseCase for ListBranches<V, C> {
    type Request = ListBranchesRequest;
    type Response = BranchList;
    type Error = ListBranchesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        refresh_remote_branches(&self.version_control, &self.credentials, &dir)?;
        Ok(BranchList {
            current: self.version_control.current_branch(&dir)?,
            branches: self.version_control.branches(&dir)?,
        })
    }
}

pub(crate) fn refresh_remote_branches<V: VersionControl, C: CredentialStore>(
    version_control: &V,
    credentials: &C,
    dir: &WorkingDirectory,
) -> Result<(), ListBranchesError> {
    let Some(remote) = version_control.remote(dir)? else {
        return Ok(());
    };
    let credentials = Credentials::from(credentials.load()?);
    match version_control.fetch_branches(dir, &remote, &credentials) {
        Ok(()) | Err(VersionControlError::Network(_)) => Ok(()),
        Err(error) => Err(error.into()),
    }
}

use std::collections::BTreeSet;

use crate::domain::port::Credentials;
use crate::domain::service::{ConflictResolution, FileChange};
use crate::domain::value_object::{
    BranchName, CommitId, CommitMessage, RemoteUrl, TrackedPath, WorkingDirectory,
};

pub trait VersionControl {
    fn is_repository(&self, dir: &WorkingDirectory) -> Result<bool, VersionControlError>;
    fn init(&self, dir: &WorkingDirectory, branch: &BranchName) -> Result<(), VersionControlError>;
    fn clone_from(
        &self,
        url: &RemoteUrl,
        into: &WorkingDirectory,
        credentials: &Credentials,
    ) -> Result<(), VersionControlError>;
    fn current_branch(&self, dir: &WorkingDirectory) -> Result<BranchName, VersionControlError>;
    fn remote(&self, dir: &WorkingDirectory) -> Result<Option<RemoteUrl>, VersionControlError>;
    fn set_remote(
        &self,
        dir: &WorkingDirectory,
        url: &RemoteUrl,
    ) -> Result<(), VersionControlError>;
    fn commit_all(
        &self,
        dir: &WorkingDirectory,
        message: &CommitMessage,
    ) -> Result<Option<CommitId>, VersionControlError>;
    fn fetch(
        &self,
        dir: &WorkingDirectory,
        url: &RemoteUrl,
        branch: &BranchName,
        credentials: &Credentials,
    ) -> Result<(), VersionControlError>;
    fn merge_remote(
        &self,
        dir: &WorkingDirectory,
        branch: &BranchName,
    ) -> Result<MergeOutcome, VersionControlError>;
    fn existing_paths(
        &self,
        dir: &WorkingDirectory,
    ) -> Result<BTreeSet<TrackedPath>, VersionControlError>;
    fn conclude_merge(
        &self,
        dir: &WorkingDirectory,
        branch: &BranchName,
        resolutions: &[ResolvedConflict],
        message: &CommitMessage,
    ) -> Result<CommitId, VersionControlError>;
    fn push(
        &self,
        dir: &WorkingDirectory,
        url: &RemoteUrl,
        branch: &BranchName,
        credentials: &Credentials,
    ) -> Result<PushOutcome, VersionControlError>;
    fn status(&self, dir: &WorkingDirectory) -> Result<RepositoryStatus, VersionControlError>;
    fn history(
        &self,
        dir: &WorkingDirectory,
        path: &TrackedPath,
        limit: usize,
    ) -> Result<Vec<Revision>, VersionControlError>;
    fn read_at(
        &self,
        dir: &WorkingDirectory,
        path: &TrackedPath,
        commit: &CommitId,
    ) -> Result<Vec<u8>, VersionControlError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VersionControlError {
    #[error("not a repository")]
    NotARepository,
    #[error("the remote requires authentication")]
    AuthenticationRequired,
    #[error("authentication with the remote failed")]
    AuthenticationFailed,
    #[error("network error: {0}")]
    Network(String),
    #[error("`{0:?}` exceeds the maximum file size")]
    FileTooLarge(TrackedPath),
    #[error("`{0:?}` not found")]
    PathNotFound(TrackedPath),
    #[error("version control failure: {0}")]
    Backend(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeOutcome {
    UpToDate,
    FastForwarded {
        changed: Vec<TrackedPath>,
    },
    Diverged {
        changed: Vec<TrackedPath>,
        conflicts: Vec<Conflict>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    pub path: TrackedPath,
    pub local: FileChange,
    pub remote: FileChange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConflict {
    pub path: TrackedPath,
    pub resolution: ConflictResolution,
    pub copy: Option<TrackedPath>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PushOutcome {
    Pushed,
    RemoteMoved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryStatus {
    pub branch: BranchName,
    pub remote: Option<RemoteUrl>,
    pub pending_changes: usize,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Revision {
    pub id: CommitId,
    pub summary: String,
    pub author: String,
    pub epoch_seconds: i64,
}

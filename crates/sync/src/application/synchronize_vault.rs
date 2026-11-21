use jade_shared::application::use_case::UseCase;

use crate::domain::port::{
    Clock, Conflict, CredentialStore, CredentialStoreError, Credentials, DeviceInfo, MergeOutcome,
    PushOutcome, ResolvedConflict, VersionControl, VersionControlError,
};
use crate::domain::service::{ConflictResolution, ConflictResolutionPolicy, SyncRetryPolicy};
use crate::domain::value_object::{
    BranchName, CommitId, CommitMessage, ConflictCopyName, InvalidWorkingDirectoryError,
    TrackedPath, WorkingDirectory,
};

pub struct SynchronizeVaultRequest {
    pub working_directory: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub committed: Option<CommitId>,
    pub changed_paths: Vec<TrackedPath>,
    pub conflict_copies: Vec<ConflictCopy>,
    pub pushed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictCopy {
    pub original: TrackedPath,
    pub copy: TrackedPath,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum SynchronizeVaultError {
    #[error(transparent)]
    InvalidWorkingDirectory(#[from] InvalidWorkingDirectoryError),
    #[error(transparent)]
    VersionControl(#[from] VersionControlError),
    #[error(transparent)]
    Credentials(#[from] CredentialStoreError),
    #[error("the remote kept changing while pushing")]
    PushRejected,
}

pub struct SynchronizeVault<V, C, K, D> {
    version_control: V,
    credentials: C,
    clock: K,
    device: D,
    conflict_policy: ConflictResolutionPolicy,
    retry_policy: SyncRetryPolicy,
}

impl<V, C, K, D> SynchronizeVault<V, C, K, D>
where
    V: VersionControl,
    C: CredentialStore,
    K: Clock,
    D: DeviceInfo,
{
    pub fn new(version_control: V, credentials: C, clock: K, device: D) -> Self {
        Self {
            version_control,
            credentials,
            clock,
            device,
            conflict_policy: ConflictResolutionPolicy,
            retry_policy: SyncRetryPolicy::default(),
        }
    }

    fn integrate_remote_changes(
        &self,
        dir: &WorkingDirectory,
        branch: &BranchName,
        report: &mut SyncReport,
    ) -> Result<(), SynchronizeVaultError> {
        match self.version_control.merge_remote(dir, branch)? {
            MergeOutcome::UpToDate => {}
            MergeOutcome::FastForwarded { changed } => report.changed_paths.extend(changed),
            MergeOutcome::Diverged { changed, conflicts } => {
                report.changed_paths.extend(changed);
                let resolutions = self.resolve(dir, conflicts)?;
                record_conflict_copies(&resolutions, report);
                self.version_control.conclude_merge(
                    dir,
                    branch,
                    &resolutions,
                    &CommitMessage::merge_remote_changes(),
                )?;
            }
        }
        Ok(())
    }

    fn resolve(
        &self,
        dir: &WorkingDirectory,
        conflicts: Vec<Conflict>,
    ) -> Result<Vec<ResolvedConflict>, SynchronizeVaultError> {
        if conflicts.is_empty() {
            return Ok(Vec::new());
        }
        let mut taken = self.version_control.existing_paths(dir)?;
        let now = self.clock.now();
        let mut resolutions = Vec::with_capacity(conflicts.len());
        for conflict in conflicts {
            let resolution = self
                .conflict_policy
                .resolve(conflict.local, conflict.remote);
            let copy = if resolution == ConflictResolution::KeepLocalAndCopyRemote {
                let name =
                    ConflictCopyName::for_conflict(&conflict.path, &now, |p| taken.contains(p));
                taken.insert(name.path().clone());
                Some(name.path().clone())
            } else {
                None
            };
            resolutions.push(ResolvedConflict {
                path: conflict.path,
                resolution,
                copy,
            });
        }
        Ok(resolutions)
    }
}

impl<V, C, K, D> UseCase for SynchronizeVault<V, C, K, D>
where
    V: VersionControl,
    C: CredentialStore,
    K: Clock,
    D: DeviceInfo,
{
    type Request = SynchronizeVaultRequest;
    type Response = SyncReport;
    type Error = SynchronizeVaultError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let dir = WorkingDirectory::parse(&request.working_directory)?;
        let message = CommitMessage::sync_from(&self.device.device_name());
        let mut report = SyncReport {
            committed: self.version_control.commit_all(&dir, &message)?,
            changed_paths: Vec::new(),
            conflict_copies: Vec::new(),
            pushed: false,
        };
        let Some(remote) = self.version_control.remote(&dir)? else {
            return Ok(report);
        };
        let branch = self.version_control.current_branch(&dir)?;
        let credentials = Credentials::from(self.credentials.load()?);
        let mut attempts = 0;
        loop {
            attempts += 1;
            self.version_control
                .fetch(&dir, &remote, &branch, &credentials)?;
            self.integrate_remote_changes(&dir, &branch, &mut report)?;
            match self
                .version_control
                .push(&dir, &remote, &branch, &credentials)?
            {
                PushOutcome::Pushed => {
                    report.pushed = true;
                    return Ok(report);
                }
                PushOutcome::RemoteMoved if self.retry_policy.allows_another_attempt(attempts) => {}
                PushOutcome::RemoteMoved => return Err(SynchronizeVaultError::PushRejected),
            }
        }
    }
}

fn record_conflict_copies(resolutions: &[ResolvedConflict], report: &mut SyncReport) {
    for resolution in resolutions {
        if let Some(copy) = &resolution.copy {
            report.changed_paths.push(copy.clone());
            report.conflict_copies.push(ConflictCopy {
                original: resolution.path.clone(),
                copy: copy.clone(),
            });
        }
    }
}

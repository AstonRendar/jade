use std::collections::BTreeSet;

use jade_sync::domain::port::{
    Clock, CredentialStore, CredentialStoreError, Credentials, DeviceInfo, MergeOutcome,
    PushOutcome, RepositoryStatus, ResolvedConflict, Revision, VersionControl, VersionControlError,
};
use jade_sync::domain::value_object::{
    AccessToken, BranchName, CommitId, CommitMessage, DeviceName, RemoteUrl, Timestamp,
    TrackedPath, WorkingDirectory,
};
use mockall::mock;

mock! {
    pub VersionControl {}

    impl VersionControl for VersionControl {
        fn is_repository(&self, dir: &WorkingDirectory) -> Result<bool, VersionControlError>;
        fn init(&self, dir: &WorkingDirectory, branch: &BranchName) -> Result<(), VersionControlError>;
        fn clone_from(&self, url: &RemoteUrl, into: &WorkingDirectory, credentials: &Credentials) -> Result<(), VersionControlError>;
        fn current_branch(&self, dir: &WorkingDirectory) -> Result<BranchName, VersionControlError>;
        fn remote(&self, dir: &WorkingDirectory) -> Result<Option<RemoteUrl>, VersionControlError>;
        fn set_remote(&self, dir: &WorkingDirectory, url: &RemoteUrl) -> Result<(), VersionControlError>;
        fn commit_all(&self, dir: &WorkingDirectory, message: &CommitMessage) -> Result<Option<CommitId>, VersionControlError>;
        fn fetch(&self, dir: &WorkingDirectory, url: &RemoteUrl, branch: &BranchName, credentials: &Credentials) -> Result<(), VersionControlError>;
        fn merge_remote(&self, dir: &WorkingDirectory, branch: &BranchName) -> Result<MergeOutcome, VersionControlError>;
        fn existing_paths(&self, dir: &WorkingDirectory) -> Result<BTreeSet<TrackedPath>, VersionControlError>;
        fn conclude_merge(&self, dir: &WorkingDirectory, branch: &BranchName, resolutions: &[ResolvedConflict], message: &CommitMessage) -> Result<CommitId, VersionControlError>;
        fn push(&self, dir: &WorkingDirectory, url: &RemoteUrl, branch: &BranchName, credentials: &Credentials) -> Result<PushOutcome, VersionControlError>;
        fn status(&self, dir: &WorkingDirectory) -> Result<RepositoryStatus, VersionControlError>;
        fn history(&self, dir: &WorkingDirectory, path: &TrackedPath, limit: usize) -> Result<Vec<Revision>, VersionControlError>;
        fn read_at(&self, dir: &WorkingDirectory, path: &TrackedPath, commit: &CommitId) -> Result<Vec<u8>, VersionControlError>;
    }
}

mock! {
    pub CredentialStore {}

    impl CredentialStore for CredentialStore {
        fn load(&self) -> Result<Option<AccessToken>, CredentialStoreError>;
        fn save(&self, token: &AccessToken) -> Result<(), CredentialStoreError>;
        fn delete(&self) -> Result<(), CredentialStoreError>;
    }
}

mock! {
    pub Clock {}

    impl Clock for Clock {
        fn now(&self) -> Timestamp;
    }
}

mock! {
    pub DeviceInfo {}

    impl DeviceInfo for DeviceInfo {
        fn device_name(&self) -> DeviceName;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChange {
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    KeepLocal,
    KeepRemote,
    KeepLocalAndCopyRemote,
    Delete,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ConflictResolutionPolicy;

impl ConflictResolutionPolicy {
    pub fn resolve(&self, local: FileChange, remote: FileChange) -> ConflictResolution {
        match (local, remote) {
            (FileChange::Modified, FileChange::Modified) => {
                ConflictResolution::KeepLocalAndCopyRemote
            }
            (FileChange::Modified, FileChange::Deleted) => ConflictResolution::KeepLocal,
            (FileChange::Deleted, FileChange::Modified) => ConflictResolution::KeepRemote,
            (FileChange::Deleted, FileChange::Deleted) => ConflictResolution::Delete,
        }
    }
}

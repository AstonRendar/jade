use jade_sync::domain::service::{ConflictResolution, ConflictResolutionPolicy, FileChange};

#[test]
fn keeps_local_and_copies_remote_when_both_sides_modified_the_file() {
    // Arrange
    let policy = ConflictResolutionPolicy;

    // Act
    let resolution = policy.resolve(FileChange::Modified, FileChange::Modified);

    // Assert
    assert_eq!(resolution, ConflictResolution::KeepLocalAndCopyRemote);
}

#[test]
fn keeps_local_when_remote_deleted_a_locally_modified_file() {
    // Arrange
    let policy = ConflictResolutionPolicy;

    // Act
    let resolution = policy.resolve(FileChange::Modified, FileChange::Deleted);

    // Assert
    assert_eq!(resolution, ConflictResolution::KeepLocal);
}

#[test]
fn keeps_remote_when_local_deleted_a_remotely_modified_file() {
    // Arrange
    let policy = ConflictResolutionPolicy;

    // Act
    let resolution = policy.resolve(FileChange::Deleted, FileChange::Modified);

    // Assert
    assert_eq!(resolution, ConflictResolution::KeepRemote);
}

#[test]
fn deletes_when_both_sides_deleted_the_file() {
    // Arrange
    let policy = ConflictResolutionPolicy;

    // Act
    let resolution = policy.resolve(FileChange::Deleted, FileChange::Deleted);

    // Assert
    assert_eq!(resolution, ConflictResolution::Delete);
}

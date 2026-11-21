use jade_shared::application::use_case::UseCase;
use jade_sync::application::list_note_history::{
    ListNoteHistory, ListNoteHistoryError, ListNoteHistoryRequest,
};
use jade_sync::application::read_note_revision::{
    ReadNoteRevision, ReadNoteRevisionError, ReadNoteRevisionRequest,
};
use jade_sync::domain::port::Revision;
use mockall::predicate::eq;

use crate::support::mocks::MockVersionControl;
use crate::support::mothers::{SyncMother, TrackedPathMother, VAULT_DIR};

#[test]
fn lists_the_revisions_of_a_note() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_history(&mut version_control, vec![revision()]);

    // Act
    let revisions = ListNoteHistory::new(version_control).execute(history_request("ideas.md"));

    // Assert
    assert_eq!(revisions, Ok(vec![revision()]));
}

#[test]
fn rejects_history_of_a_path_outside_the_vault() {
    // Arrange
    let version_control = MockVersionControl::new();

    // Act
    let result = ListNoteHistory::new(version_control).execute(history_request("../x.md"));

    // Assert
    assert!(matches!(result, Err(ListNoteHistoryError::InvalidPath(_))));
}

#[test]
fn reads_a_note_as_it_was_in_a_revision() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_content_at_revision(&mut version_control, b"old text".to_vec());

    // Act
    let content = ReadNoteRevision::new(version_control).execute(revision_request(&"a".repeat(40)));

    // Assert
    assert_eq!(content, Ok(b"old text".to_vec()));
}

#[test]
fn rejects_an_invalid_revision_id() {
    // Arrange
    let version_control = MockVersionControl::new();

    // Act
    let result = ReadNoteRevision::new(version_control).execute(revision_request("nope"));

    // Assert
    assert!(matches!(
        result,
        Err(ReadNoteRevisionError::InvalidCommitId(_))
    ));
}

fn history_request(path: &str) -> ListNoteHistoryRequest {
    ListNoteHistoryRequest {
        working_directory: VAULT_DIR.to_string(),
        path: path.to_string(),
        limit: 50,
    }
}

fn revision_request(commit: &str) -> ReadNoteRevisionRequest {
    ReadNoteRevisionRequest {
        working_directory: VAULT_DIR.to_string(),
        path: "ideas.md".to_string(),
        commit: commit.to_string(),
    }
}

fn revision() -> Revision {
    Revision {
        id: SyncMother::commit_id(),
        summary: "Sync from laptop".to_string(),
        author: "Example Author".to_string(),
        epoch_seconds: 1_795_000_000,
    }
}

fn given_history(version_control: &mut MockVersionControl, revisions: Vec<Revision>) {
    version_control
        .expect_history()
        .with(
            eq(SyncMother::working_directory()),
            eq(TrackedPathMother::of("ideas.md")),
            eq(50),
        )
        .return_once(move |_, _, _| Ok(revisions));
}

fn given_content_at_revision(version_control: &mut MockVersionControl, content: Vec<u8>) {
    version_control
        .expect_read_at()
        .with(
            eq(SyncMother::working_directory()),
            eq(TrackedPathMother::of("ideas.md")),
            eq(SyncMother::commit_id()),
        )
        .return_once(move |_, _, _| Ok(content));
}

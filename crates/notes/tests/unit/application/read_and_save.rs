use jade_notes::application::error::NotesError;
use jade_notes::application::list_vault_entries::{ListVaultEntries, ListVaultEntriesRequest};
use jade_notes::application::read_note::{ReadNote, ReadNoteRequest};
use jade_notes::application::save_note::{SaveNote, SaveNoteRequest};
use jade_notes::domain::model::{EntryKind, VaultEntry};
use jade_shared::application::use_case::UseCase;
use mockall::predicate::eq;

use crate::support::mocks::MockNoteRepository;
use crate::support::mothers::{NoteMother, VAULT};

#[test]
fn reads_a_note() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_read()
        .with(eq(NoteMother::root()), eq(NoteMother::path("ideas.md")))
        .returning(|_, _| Ok("text".to_string()));

    // Act
    let content = ReadNote::new(repository).execute(ReadNoteRequest {
        vault_root: VAULT.to_string(),
        path: "ideas.md".to_string(),
    });

    // Assert
    assert_eq!(content, Ok("text".to_string()));
}

#[test]
fn saves_a_note() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_write()
        .with(
            eq(NoteMother::root()),
            eq(NoteMother::path("ideas.md")),
            eq("new"),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));

    // Act
    let result = SaveNote::new(repository).execute(save_request("ideas.md"));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn only_saves_markdown_notes() {
    // Arrange
    let repository = MockNoteRepository::new();

    // Act
    let result = SaveNote::new(repository).execute(save_request("diagram.png"));

    // Assert
    assert!(matches!(result, Err(NotesError::InvalidPath(_))));
}

#[test]
fn lists_entries_sorted_by_path() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository.expect_list().returning(|_| {
        Ok(vec![
            entry("b.md", EntryKind::Note),
            entry("a", EntryKind::Folder),
        ])
    });

    // Act
    let entries = ListVaultEntries::new(repository).execute(ListVaultEntriesRequest {
        vault_root: VAULT.to_string(),
    });

    // Assert
    assert_eq!(
        entries,
        Ok(vec![
            entry("a", EntryKind::Folder),
            entry("b.md", EntryKind::Note)
        ])
    );
}

fn save_request(path: &str) -> SaveNoteRequest {
    SaveNoteRequest {
        vault_root: VAULT.to_string(),
        path: path.to_string(),
        content: "new".to_string(),
    }
}

fn entry(path: &str, kind: EntryKind) -> VaultEntry {
    VaultEntry {
        path: NoteMother::vault_path(path),
        kind,
    }
}

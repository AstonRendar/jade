use jade_notes::application::delete_entry::{DeleteEntry, DeleteEntryRequest};
use jade_notes::application::error::NotesError;
use jade_notes::domain::port::NoteRepositoryError;
use jade_shared::application::use_case::UseCase;
use mockall::predicate::eq;

use crate::support::mocks::MockNoteRepository;
use crate::support::mothers::{NoteMother, VAULT};

#[test]
fn deletes_an_entry() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    expect_deleted(&mut repository, "projects");

    // Act
    let result = DeleteEntry::new(repository).execute(request("projects"));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn reports_storage_failures() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_delete()
        .returning(|_, _| Err(NoteRepositoryError::Io("disk full".to_string())));

    // Act
    let result = DeleteEntry::new(repository).execute(request("projects"));

    // Assert
    assert_eq!(result, Err(NotesError::Storage("disk full".to_string())));
}

fn request(path: &str) -> DeleteEntryRequest {
    DeleteEntryRequest {
        vault_root: VAULT.to_string(),
        path: path.to_string(),
    }
}

fn expect_deleted(repository: &mut MockNoteRepository, path: &str) {
    repository
        .expect_delete()
        .with(eq(NoteMother::root()), eq(NoteMother::vault_path(path)))
        .times(1)
        .returning(|_, _| Ok(()));
}

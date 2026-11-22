use jade_notes::application::error::NotesError;
use jade_notes::application::rename_entry::{RenameEntry, RenameEntryRequest};
use jade_notes::domain::port::NoteRepositoryError;
use jade_shared::application::use_case::UseCase;
use mockall::predicate::eq;

use crate::support::mocks::MockNoteRepository;
use crate::support::mothers::{NoteMother, VAULT};

#[test]
fn renames_an_entry() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_target_exists(&mut repository, false);
    expect_renamed(&mut repository, "ideas.md", "thoughts.md");

    // Act
    let renamed = RenameEntry::new(repository).execute(request("ideas.md", "thoughts.md"));

    // Assert
    assert_eq!(renamed, Ok(NoteMother::vault_path("thoughts.md")));
}

#[test]
fn refuses_to_replace_an_existing_entry() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_target_exists(&mut repository, true);

    // Act
    let result = RenameEntry::new(repository).execute(request("a.md", "b.md"));

    // Assert
    assert_eq!(
        result,
        Err(NotesError::AlreadyExists(NoteMother::vault_path("b.md")))
    );
}

#[test]
fn refuses_to_move_a_folder_inside_itself() {
    // Arrange
    let repository = MockNoteRepository::new();

    // Act
    let result = RenameEntry::new(repository).execute(request("projects", "projects/inner"));

    // Assert
    assert_eq!(result, Err(NotesError::InvalidMove));
}

#[test]
fn reports_a_missing_source() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_target_exists(&mut repository, false);
    given_rename_fails_with_missing_source(&mut repository);

    // Act
    let result = RenameEntry::new(repository).execute(request("gone.md", "new.md"));

    // Assert
    assert_eq!(
        result,
        Err(NotesError::NotFound(NoteMother::vault_path("gone.md")))
    );
}

fn request(from: &str, to: &str) -> RenameEntryRequest {
    RenameEntryRequest {
        vault_root: VAULT.to_string(),
        from: from.to_string(),
        to: to.to_string(),
    }
}

fn given_target_exists(repository: &mut MockNoteRepository, exists: bool) {
    repository
        .expect_entry_exists()
        .returning(move |_, _| Ok(exists));
}

fn given_rename_fails_with_missing_source(repository: &mut MockNoteRepository) {
    repository
        .expect_rename()
        .returning(|_, from, _| Err(NoteRepositoryError::NotFound(from.clone())));
}

fn expect_renamed(repository: &mut MockNoteRepository, from: &str, to: &str) {
    repository
        .expect_rename()
        .with(
            eq(NoteMother::root()),
            eq(NoteMother::vault_path(from)),
            eq(NoteMother::vault_path(to)),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
}

use jade_notes::application::create_folder::{CreateFolder, CreateFolderRequest};
use jade_notes::application::create_note::{CreateNote, CreateNoteRequest};
use jade_notes::application::error::NotesError;
use jade_shared::application::use_case::UseCase;
use mockall::predicate::eq;

use crate::support::mocks::MockNoteRepository;
use crate::support::mothers::{NoteMother, VAULT};

#[test]
fn creates_an_empty_note_inside_a_folder() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_entry_exists(&mut repository, false);
    expect_empty_note_written(&mut repository, "projects/Plan.md");

    // Act
    let created = CreateNote::new(repository).execute(note_request(Some("projects"), "Plan"));

    // Assert
    assert_eq!(created, Ok(NoteMother::path("projects/Plan.md")));
}

#[test]
fn refuses_to_overwrite_an_existing_note() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_entry_exists(&mut repository, true);

    // Act
    let result = CreateNote::new(repository).execute(note_request(None, "Ideas"));

    // Assert
    assert_eq!(
        result,
        Err(NotesError::AlreadyExists(NoteMother::vault_path(
            "Ideas.md"
        )))
    );
}

#[test]
fn rejects_titles_with_separators() {
    // Arrange
    let repository = MockNoteRepository::new();

    // Act
    let result = CreateNote::new(repository).execute(note_request(None, "a/b"));

    // Assert
    assert!(matches!(result, Err(NotesError::InvalidPath(_))));
}

#[test]
fn creates_a_folder() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_entry_exists(&mut repository, false);
    expect_folder_created(&mut repository, "projects/2026");

    // Act
    let created = CreateFolder::new(repository).execute(folder_request("projects/2026"));

    // Assert
    assert_eq!(created, Ok(NoteMother::vault_path("projects/2026")));
}

#[test]
fn refuses_to_create_an_existing_folder() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_entry_exists(&mut repository, true);

    // Act
    let result = CreateFolder::new(repository).execute(folder_request("projects"));

    // Assert
    assert_eq!(
        result,
        Err(NotesError::AlreadyExists(NoteMother::vault_path(
            "projects"
        )))
    );
}

fn note_request(folder: Option<&str>, title: &str) -> CreateNoteRequest {
    CreateNoteRequest {
        vault_root: VAULT.to_string(),
        folder: folder.map(str::to_string),
        title: title.to_string(),
    }
}

fn folder_request(path: &str) -> CreateFolderRequest {
    CreateFolderRequest {
        vault_root: VAULT.to_string(),
        path: path.to_string(),
    }
}

fn given_entry_exists(repository: &mut MockNoteRepository, exists: bool) {
    repository
        .expect_entry_exists()
        .returning(move |_, _| Ok(exists));
}

fn expect_empty_note_written(repository: &mut MockNoteRepository, path: &str) {
    repository
        .expect_write()
        .with(eq(NoteMother::root()), eq(NoteMother::path(path)), eq(""))
        .times(1)
        .returning(|_, _, _| Ok(()));
}

fn expect_folder_created(repository: &mut MockNoteRepository, path: &str) {
    repository
        .expect_create_folder()
        .with(eq(NoteMother::root()), eq(NoteMother::vault_path(path)))
        .times(1)
        .returning(|_, _| Ok(()));
}

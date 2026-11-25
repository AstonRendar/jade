use jade_notes::application::error::NotesError;
use jade_notes::application::follow_link::{FollowLink, FollowLinkRequest, FollowedLink};
use jade_notes::domain::model::{EntryKind, VaultEntry};
use jade_shared::application::use_case::UseCase;
use mockall::predicate::eq;

use crate::support::mocks::MockNoteRepository;
use crate::support::mothers::{NoteMother, VAULT};

#[test]
fn opens_the_note_a_link_resolves_to() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_notes(&mut repository, &["projects/Plan.md"]);

    // Act
    let followed = FollowLink::new(repository).execute(request("plan"));

    // Assert
    assert_eq!(
        followed,
        Ok(FollowedLink {
            note: NoteMother::path("projects/Plan.md"),
            created: false,
        })
    );
}

#[test]
fn creates_the_note_when_the_link_is_unresolved() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_notes(&mut repository, &[]);
    expect_empty_note_written(&mut repository, "ideas/Someday.md");

    // Act
    let followed = FollowLink::new(repository).execute(request("ideas/Someday"));

    // Assert
    assert_eq!(
        followed,
        Ok(FollowedLink {
            note: NoteMother::path("ideas/Someday.md"),
            created: true,
        })
    );
}

#[test]
fn rejects_targets_that_cannot_be_a_note() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_notes(&mut repository, &[]);

    // Act
    let result = FollowLink::new(repository).execute(request("../outside"));

    // Assert
    assert!(matches!(result, Err(NotesError::InvalidPath(_))));
}

fn request(target: &str) -> FollowLinkRequest {
    FollowLinkRequest {
        vault_root: VAULT.to_string(),
        target: target.to_string(),
    }
}

fn given_notes(repository: &mut MockNoteRepository, paths: &[&str]) {
    let entries: Vec<_> = paths
        .iter()
        .map(|path| VaultEntry {
            path: NoteMother::vault_path(path),
            kind: EntryKind::Note,
        })
        .chain([VaultEntry {
            path: NoteMother::vault_path("image.png"),
            kind: EntryKind::File,
        }])
        .collect();
    repository.expect_list().return_once(move |_| Ok(entries));
}

fn expect_empty_note_written(repository: &mut MockNoteRepository, path: &str) {
    repository
        .expect_write()
        .with(eq(NoteMother::root()), eq(NoteMother::path(path)), eq(""))
        .times(1)
        .returning(|_, _, _| Ok(()));
}

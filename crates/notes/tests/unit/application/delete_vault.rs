use jade_notes::application::delete_vault::{DeleteVault, DeleteVaultRequest};
use jade_notes::application::error::NotesError;
use jade_notes::domain::value_object::VaultRoot;
use jade_shared::application::use_case::UseCase;
use mockall::predicate::eq;

use crate::support::mocks::MockNoteRepository;

#[test]
fn deletes_a_vault_kept_in_the_container() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository.expect_vault_exists().return_const(true);
    expect_root_deleted(&mut repository, "/data/vaults/notes");

    // Act
    let result =
        DeleteVault::new(repository).execute(request("/data/vaults/notes", "/data/vaults"));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn refuses_a_vault_outside_the_container() {
    // Arrange
    let vaults = ["/home/user/notes", "/data/vaults", "/data/vaults/a/b"];

    // Act
    let results: Vec<_> = vaults
        .into_iter()
        .map(|vault| {
            DeleteVault::new(given_nothing_is_deleted()).execute(request(vault, "/data/vaults"))
        })
        .collect();

    // Assert
    assert!(results
        .iter()
        .all(|result| matches!(result, Err(NotesError::InvalidPath(_)))));
}

#[test]
fn reports_a_vault_that_no_longer_exists() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository.expect_vault_exists().return_const(false);

    // Act
    let result =
        DeleteVault::new(repository).execute(request("/data/vaults/notes", "/data/vaults"));

    // Assert
    assert_eq!(result, Err(NotesError::VaultNotFound));
}

fn request(vault_root: &str, container: &str) -> DeleteVaultRequest {
    DeleteVaultRequest {
        vault_root: vault_root.to_string(),
        container: container.to_string(),
    }
}

fn given_nothing_is_deleted() -> MockNoteRepository {
    let mut repository = MockNoteRepository::new();
    repository.expect_vault_exists().return_const(true);
    repository.expect_delete_root().never();
    repository
}

fn expect_root_deleted(repository: &mut MockNoteRepository, path: &str) {
    repository
        .expect_delete_root()
        .with(eq(VaultRoot::parse(path).expect("valid root")))
        .times(1)
        .returning(|_| Ok(()));
}

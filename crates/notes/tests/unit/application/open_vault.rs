use jade_notes::application::error::NotesError;
use jade_notes::application::open_vault::{OpenVault, OpenVaultRequest, VaultInfo};
use jade_shared::application::use_case::UseCase;

use crate::support::mocks::MockNoteRepository;
use crate::support::mothers::{NoteMother, VAULT};

#[test]
fn opens_an_existing_folder() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_vault_exists(&mut repository, true);

    // Act
    let info = OpenVault::new(repository).execute(request(VAULT));

    // Assert
    assert_eq!(
        info,
        Ok(VaultInfo {
            root: NoteMother::root(),
            name: "notes".to_string(),
        })
    );
}

#[test]
fn fails_when_the_folder_does_not_exist() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_vault_exists(&mut repository, false);

    // Act
    let result = OpenVault::new(repository).execute(request(VAULT));

    // Assert
    assert_eq!(result, Err(NotesError::VaultNotFound));
}

#[test]
fn rejects_relative_folders() {
    // Arrange
    let repository = MockNoteRepository::new();

    // Act
    let result = OpenVault::new(repository).execute(request("notes"));

    // Assert
    assert!(matches!(result, Err(NotesError::InvalidVaultRoot(_))));
}

fn request(root: &str) -> OpenVaultRequest {
    OpenVaultRequest {
        vault_root: root.to_string(),
    }
}

fn given_vault_exists(repository: &mut MockNoteRepository, exists: bool) {
    repository.expect_vault_exists().return_const(exists);
}

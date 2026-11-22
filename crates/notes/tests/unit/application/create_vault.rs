use jade_notes::application::create_vault::{CreateVault, CreateVaultRequest};
use jade_notes::application::error::NotesError;
use jade_notes::application::open_vault::VaultInfo;
use jade_notes::domain::value_object::VaultRoot;
use jade_shared::application::use_case::UseCase;
use mockall::predicate::eq;

use crate::support::mocks::MockNoteRepository;

#[test]
fn creates_the_vault_folder_inside_the_parent() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository.expect_vault_exists().return_const(false);
    expect_root_created(&mut repository, "/vaults/Journal");

    // Act
    let created = CreateVault::new(repository).execute(request("/vaults", "Journal"));

    // Assert
    assert_eq!(
        created,
        Ok(VaultInfo {
            root: root("/vaults/Journal"),
            name: "Journal".to_string(),
        })
    );
}

#[test]
fn refuses_an_existing_folder() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository.expect_vault_exists().return_const(true);

    // Act
    let result = CreateVault::new(repository).execute(request("/vaults", "Journal"));

    // Assert
    assert!(matches!(result, Err(NotesError::AlreadyExists(_))));
}

#[test]
fn rejects_names_that_are_not_a_single_folder() {
    // Arrange
    let names = ["", "  ", "a/b", "..", ".hidden"];

    // Act
    let accepted: Vec<&str> = names
        .into_iter()
        .filter(|name| {
            CreateVault::new(MockNoteRepository::new())
                .execute(request("/vaults", name))
                .is_ok()
        })
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

fn request(parent: &str, name: &str) -> CreateVaultRequest {
    CreateVaultRequest {
        parent: parent.to_string(),
        name: name.to_string(),
    }
}

fn root(path: &str) -> VaultRoot {
    VaultRoot::parse(path).expect("valid root")
}

fn expect_root_created(repository: &mut MockNoteRepository, path: &str) {
    repository
        .expect_create_root()
        .with(eq(root(path)))
        .times(1)
        .returning(|_| Ok(()));
}

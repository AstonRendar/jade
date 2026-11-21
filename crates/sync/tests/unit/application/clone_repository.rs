use jade_shared::application::use_case::UseCase;
use jade_sync::application::clone_repository::{
    CloneRepository, CloneRepositoryError, CloneRepositoryRequest,
};
use jade_sync::domain::port::Credentials;
use mockall::predicate::eq;

use crate::support::mocks::{MockCredentialStore, MockVersionControl};
use crate::support::mothers::{SyncMother, REMOTE, VAULT_DIR};

#[test]
fn clones_with_the_stored_token() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    let mut credentials = MockCredentialStore::new();
    given_a_stored_token(&mut credentials);
    expect_clone_with(
        &mut version_control,
        Credentials::Token(SyncMother::token()),
    );

    // Act
    let result = CloneRepository::new(version_control, credentials).execute(request(REMOTE));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn clones_anonymously_without_a_stored_token() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    let mut credentials = MockCredentialStore::new();
    credentials.expect_load().returning(|| Ok(None));
    expect_clone_with(&mut version_control, Credentials::Anonymous);

    // Act
    let result = CloneRepository::new(version_control, credentials).execute(request(REMOTE));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_an_unsupported_url() {
    // Arrange
    let version_control = MockVersionControl::new();
    let credentials = MockCredentialStore::new();

    // Act
    let result =
        CloneRepository::new(version_control, credentials).execute(request("ftp://x/y.git"));

    // Assert
    assert!(matches!(
        result,
        Err(CloneRepositoryError::InvalidRemoteUrl(_))
    ));
}

fn request(url: &str) -> CloneRepositoryRequest {
    CloneRepositoryRequest {
        url: url.to_string(),
        working_directory: VAULT_DIR.to_string(),
    }
}

fn given_a_stored_token(credentials: &mut MockCredentialStore) {
    credentials
        .expect_load()
        .returning(|| Ok(Some(SyncMother::token())));
}

fn expect_clone_with(version_control: &mut MockVersionControl, credentials: Credentials) {
    version_control
        .expect_clone_from()
        .with(
            eq(SyncMother::remote()),
            eq(SyncMother::working_directory()),
            eq(credentials),
        )
        .times(1)
        .returning(|_, _, _| Ok(()));
}

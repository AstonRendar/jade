use jade_shared::application::use_case::UseCase;
use jade_sync::application::check_access_token::CheckAccessToken;
use jade_sync::application::store_access_token::{
    StoreAccessToken, StoreAccessTokenError, StoreAccessTokenRequest,
};
use mockall::predicate::eq;

use crate::support::mocks::MockCredentialStore;
use crate::support::mothers::SyncMother;

#[test]
fn stores_a_token() {
    // Arrange
    let mut credentials = MockCredentialStore::new();
    expect_token_saved(&mut credentials);

    // Act
    let result = StoreAccessToken::new(credentials).execute(request("token-123"));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn deletes_the_token_when_given_an_empty_one() {
    // Arrange
    let mut credentials = MockCredentialStore::new();
    expect_token_deleted(&mut credentials);

    // Act
    let result = StoreAccessToken::new(credentials).execute(request(""));

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_a_malformed_token() {
    // Arrange
    let credentials = MockCredentialStore::new();

    // Act
    let result = StoreAccessToken::new(credentials).execute(request("has space"));

    // Assert
    assert!(matches!(
        result,
        Err(StoreAccessTokenError::InvalidToken(_))
    ));
}

#[test]
fn tells_whether_a_token_is_stored() {
    // Arrange
    let mut credentials = MockCredentialStore::new();
    credentials
        .expect_load()
        .returning(|| Ok(Some(SyncMother::token())));

    // Act
    let has_token = CheckAccessToken::new(credentials).execute(());

    // Assert
    assert_eq!(has_token, Ok(true));
}

fn request(token: &str) -> StoreAccessTokenRequest {
    StoreAccessTokenRequest {
        token: token.to_string(),
    }
}

fn expect_token_saved(credentials: &mut MockCredentialStore) {
    credentials
        .expect_save()
        .with(eq(SyncMother::token()))
        .times(1)
        .returning(|_| Ok(()));
}

fn expect_token_deleted(credentials: &mut MockCredentialStore) {
    credentials.expect_delete().times(1).returning(|| Ok(()));
}

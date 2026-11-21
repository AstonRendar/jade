use jade_sync::domain::port::CredentialStore;
use jade_sync::infrastructure::InMemoryCredentialStore;

use crate::support::mothers::SyncMother;

#[test]
fn starts_without_a_token() {
    // Arrange
    let store = InMemoryCredentialStore::default();

    // Act
    let token = store.load();

    // Assert
    assert_eq!(token, Ok(None));
}

#[test]
fn returns_the_saved_token() {
    // Arrange
    let store = InMemoryCredentialStore::default();
    store.save(&SyncMother::token()).expect("token is saved");

    // Act
    let token = store.load();

    // Assert
    assert_eq!(token, Ok(Some(SyncMother::token())));
}

#[test]
fn forgets_a_deleted_token() {
    // Arrange
    let store = InMemoryCredentialStore::default();
    store.save(&SyncMother::token()).expect("token is saved");
    store.delete().expect("token is deleted");

    // Act
    let token = store.load();

    // Assert
    assert_eq!(token, Ok(None));
}

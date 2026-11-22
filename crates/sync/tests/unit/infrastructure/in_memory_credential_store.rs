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

#[test]
fn can_be_shared_by_reference() {
    // Arrange
    let store = InMemoryCredentialStore::default();

    // Act
    save_through(&store);

    // Assert
    assert_eq!(store.load(), Ok(Some(SyncMother::token())));
}

fn save_through(credentials: impl CredentialStore) {
    credentials
        .save(&SyncMother::token())
        .expect("token is saved");
}

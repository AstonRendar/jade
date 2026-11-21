use jade_shared::application::use_case::UseCase;
use jade_sync::application::connect_remote::{
    ConnectRemote, ConnectRemoteError, ConnectRemoteRequest,
};
use mockall::predicate::eq;

use crate::support::mocks::MockVersionControl;
use crate::support::mothers::{SyncMother, REMOTE, VAULT_DIR};

#[test]
fn sets_the_remote_of_a_repository() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_is_repository(&mut version_control, true);
    expect_remote_set(&mut version_control);

    // Act
    let result = ConnectRemote::new(version_control).execute(request());

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn requires_a_repository() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_is_repository(&mut version_control, false);

    // Act
    let result = ConnectRemote::new(version_control).execute(request());

    // Assert
    assert_eq!(result, Err(ConnectRemoteError::NotARepository));
}

fn request() -> ConnectRemoteRequest {
    ConnectRemoteRequest {
        working_directory: VAULT_DIR.to_string(),
        url: REMOTE.to_string(),
    }
}

fn given_is_repository(version_control: &mut MockVersionControl, is_repository: bool) {
    version_control
        .expect_is_repository()
        .returning(move |_| Ok(is_repository));
}

fn expect_remote_set(version_control: &mut MockVersionControl) {
    version_control
        .expect_set_remote()
        .with(
            eq(SyncMother::working_directory()),
            eq(SyncMother::remote()),
        )
        .times(1)
        .returning(|_, _| Ok(()));
}

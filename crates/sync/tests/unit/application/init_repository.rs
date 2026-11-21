use jade_shared::application::use_case::UseCase;
use jade_sync::application::init_repository::{
    InitRepository, InitRepositoryError, InitRepositoryRequest,
};
use mockall::predicate::eq;

use crate::support::mocks::MockVersionControl;
use crate::support::mothers::{SyncMother, VAULT_DIR};

#[test]
fn initialises_a_repository_on_main() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_is_repository(&mut version_control, false);
    expect_init_on_main(&mut version_control);

    // Act
    let result = InitRepository::new(version_control).execute(request());

    // Assert
    assert_eq!(result, Ok(()));
}

#[test]
fn refuses_to_initialise_an_existing_repository() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_is_repository(&mut version_control, true);

    // Act
    let result = InitRepository::new(version_control).execute(request());

    // Assert
    assert_eq!(result, Err(InitRepositoryError::AlreadyARepository));
}

fn request() -> InitRepositoryRequest {
    InitRepositoryRequest {
        working_directory: VAULT_DIR.to_string(),
    }
}

fn given_is_repository(version_control: &mut MockVersionControl, is_repository: bool) {
    version_control
        .expect_is_repository()
        .returning(move |_| Ok(is_repository));
}

fn expect_init_on_main(version_control: &mut MockVersionControl) {
    version_control
        .expect_init()
        .with(
            eq(SyncMother::working_directory()),
            eq(SyncMother::main_branch()),
        )
        .times(1)
        .returning(|_, _| Ok(()));
}

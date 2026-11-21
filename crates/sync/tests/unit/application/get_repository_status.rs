use jade_shared::application::use_case::UseCase;
use jade_sync::application::get_repository_status::{
    GetRepositoryStatus, GetRepositoryStatusRequest, RepositoryState,
};
use jade_sync::domain::port::RepositoryStatus;

use crate::support::mocks::MockVersionControl;
use crate::support::mothers::{SyncMother, VAULT_DIR};

#[test]
fn reports_a_folder_that_is_not_a_repository() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_is_repository(&mut version_control, false);

    // Act
    let state = GetRepositoryStatus::new(version_control).execute(request());

    // Assert
    assert_eq!(state, Ok(RepositoryState::NotARepository));
}

#[test]
fn reports_the_status_of_a_repository() {
    // Arrange
    let mut version_control = MockVersionControl::new();
    given_is_repository(&mut version_control, true);
    given_status(&mut version_control, status());

    // Act
    let state = GetRepositoryStatus::new(version_control).execute(request());

    // Assert
    assert_eq!(state, Ok(RepositoryState::Ready(status())));
}

fn request() -> GetRepositoryStatusRequest {
    GetRepositoryStatusRequest {
        working_directory: VAULT_DIR.to_string(),
    }
}

fn status() -> RepositoryStatus {
    RepositoryStatus {
        branch: SyncMother::main_branch(),
        remote: Some(SyncMother::remote()),
        pending_changes: 2,
        ahead: 1,
        behind: 0,
    }
}

fn given_is_repository(version_control: &mut MockVersionControl, is_repository: bool) {
    version_control
        .expect_is_repository()
        .returning(move |_| Ok(is_repository));
}

fn given_status(version_control: &mut MockVersionControl, status: RepositoryStatus) {
    version_control
        .expect_status()
        .return_once(move |_| Ok(status));
}

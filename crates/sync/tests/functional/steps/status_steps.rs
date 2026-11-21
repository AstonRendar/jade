use cucumber::{then, when};
use jade_shared::application::use_case::UseCase;
use jade_sync::application::get_repository_status::{
    GetRepositoryStatus, GetRepositoryStatusRequest, RepositoryState,
};
use jade_sync::domain::port::RepositoryStatus;
use jade_sync::infrastructure::Git2VersionControl;

use crate::world::SyncWorld;

#[when(expr = "the status of {string} is requested")]
fn status_is_requested(world: &mut SyncWorld, device: String) {
    let state = GetRepositoryStatus::new(Git2VersionControl)
        .execute(GetRepositoryStatusRequest {
            working_directory: world.device_working_directory(&device),
        })
        .expect("status is read");
    world.status = Some(state);
}

#[then(expr = "{string} has {int} pending change(s)")]
fn pending_changes(world: &mut SyncWorld, _device: String, expected: usize) {
    assert_eq!(ready(world).pending_changes, expected);
}

#[then(expr = "{string} is on branch {string} with a remote")]
fn on_branch_with_remote(world: &mut SyncWorld, _device: String, branch: String) {
    let status = ready(world);
    assert_eq!(status.branch.as_str(), branch);
    assert!(status.remote.is_some());
}

#[then(expr = "{string} is {int} commits ahead and {int} behind")]
fn ahead_and_behind(world: &mut SyncWorld, _device: String, ahead: usize, behind: usize) {
    let status = ready(world);
    assert_eq!((status.ahead, status.behind), (ahead, behind));
}

#[then(expr = "{string} is not a repository")]
fn not_a_repository(world: &mut SyncWorld, _device: String) {
    assert_eq!(world.status, Some(RepositoryState::NotARepository));
}

fn ready(world: &SyncWorld) -> &RepositoryStatus {
    match world.status.as_ref().expect("status was requested") {
        RepositoryState::Ready(status) => status,
        RepositoryState::NotARepository => panic!("not a repository"),
    }
}

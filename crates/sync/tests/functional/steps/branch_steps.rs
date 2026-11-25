use cucumber::{given, then, when};
use jade_shared::application::use_case::UseCase;
use jade_sync::application::list_branches::{ListBranches, ListBranchesRequest};
use jade_sync::application::switch_branch::{SwitchBranch, SwitchBranchError, SwitchBranchRequest};
use jade_sync::domain::port::VersionControlError;
use jade_sync::infrastructure::{Git2VersionControl, InMemoryCredentialStore};

use crate::doubles::NamedDevice;
use crate::world::SyncWorld;

#[given(expr = "the remote also has the branch {string} with {string} containing {string}")]
fn the_remote_has_a_branch(world: &mut SyncWorld, branch: String, path: String, content: String) {
    let seed = git2::Repository::open(world.device_dir("seed")).expect("seed opens");
    let head = seed
        .head()
        .and_then(|head| head.peel_to_commit())
        .expect("seed has a commit");
    seed.branch(&branch, &head, false)
        .expect("branch is created");
    seed.set_head(&format!("refs/heads/{branch}"))
        .expect("branch is checked out");
    std::fs::write(world.device_dir("seed").join(&path), content).expect("note is written");
    let mut index = seed.index().expect("index opens");
    index
        .add_path(std::path::Path::new(&path))
        .expect("note is staged");
    index.write().expect("index is written");
    let tree = seed
        .find_tree(index.write_tree().expect("tree is written"))
        .expect("tree exists");
    let signature = git2::Signature::now("Seed", "seed@example.com").expect("signature");
    seed.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Work in progress",
        &tree,
        &[&head],
    )
    .expect("commit is created");
    seed.find_remote("origin")
        .and_then(|mut remote| {
            remote.push(&[format!("refs/heads/{branch}:refs/heads/{branch}")], None)
        })
        .expect("branch is pushed");
}

#[given(expr = "{string} switches to the branch {string}")]
#[when(expr = "{string} switches to the branch {string}")]
fn a_device_switches(world: &mut SyncWorld, device: String, branch: String) {
    let result = SwitchBranch::new(
        Git2VersionControl,
        InMemoryCredentialStore::default(),
        NamedDevice(device.clone()),
    )
    .execute(SwitchBranchRequest {
        working_directory: world.device_working_directory(&device),
        branch,
    });
    world.switch_result = Some(result);
}

#[when(expr = "{string} lists its branches")]
fn a_device_lists_branches(world: &mut SyncWorld, device: String) {
    let list = ListBranches::new(Git2VersionControl, InMemoryCredentialStore::default())
        .execute(ListBranchesRequest {
            working_directory: world.device_working_directory(&device),
        })
        .expect("branches are listed");
    world.branches = Some(list);
}

#[then(expr = "{string} is on {string} and knows the branches {string}")]
fn a_device_knows_branches(world: &mut SyncWorld, _device: String, current: String, names: String) {
    let list = world.branches.as_ref().expect("branches were listed");
    assert_eq!(list.current.as_str(), current);
    let known: Vec<&str> = list.branches.iter().map(|branch| branch.as_str()).collect();
    assert_eq!(known.join(", "), names);
}

#[then("switching fails because the branch does not exist")]
fn switching_fails(world: &mut SyncWorld) {
    assert!(matches!(
        world.switch_result,
        Some(Err(SwitchBranchError::VersionControl(
            VersionControlError::BranchNotFound(_)
        )))
    ));
}

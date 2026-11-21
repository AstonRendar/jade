use cucumber::{given, then};
use jade_shared::application::use_case::UseCase;
use jade_sync::application::connect_remote::{ConnectRemote, ConnectRemoteRequest};
use jade_sync::application::init_repository::{InitRepository, InitRepositoryRequest};
use jade_sync::infrastructure::Git2VersionControl;

use crate::world::SyncWorld;

#[given("an empty remote vault")]
fn an_empty_remote_vault(world: &mut SyncWorld) {
    let dir = world.sandbox_path("remote.git");
    let mut options = git2::RepositoryInitOptions::new();
    options.bare(true).initial_head("main");
    git2::Repository::init_opts(&dir, &options).expect("bare remote is created");
    world.remote = Some(dir);
}

#[given(expr = "a remote vault with {string} containing {string}")]
fn a_remote_vault_with_a_note(world: &mut SyncWorld, path: String, content: String) {
    an_empty_remote_vault(world);
    let seed = world.add_device("seed");
    InitRepository::new(Git2VersionControl)
        .execute(InitRepositoryRequest {
            working_directory: seed.clone(),
        })
        .expect("seed repository is initialised");
    ConnectRemote::new(Git2VersionControl)
        .execute(ConnectRemoteRequest {
            working_directory: seed,
            url: world.remote_url(),
        })
        .expect("seed is connected");
    std::fs::write(world.device_dir("seed").join(path), content).expect("note is written");
    world.synchronize("seed");
}

#[then(expr = "the remote head is a merge commit {string}")]
fn the_remote_head_is_a_merge_commit(world: &mut SyncWorld, summary: String) {
    let head = world.remote_head();
    assert_eq!(head.parent_count, 2);
    assert_eq!(head.summary, summary);
}

#[then("the remote history was never rewritten")]
fn the_remote_history_was_never_rewritten(world: &mut SyncWorld) {
    let rewritten: Vec<_> = world
        .pushed_heads
        .iter()
        .filter(|head| !world.remote_descends_from(**head))
        .collect();
    assert!(rewritten.is_empty(), "lost remote heads: {rewritten:?}");
}

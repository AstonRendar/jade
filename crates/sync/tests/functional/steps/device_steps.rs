use std::fs;

use cucumber::{given, then, when};
use jade_shared::application::use_case::UseCase;
use jade_sync::application::clone_repository::{CloneRepository, CloneRepositoryRequest};
use jade_sync::application::connect_remote::{ConnectRemote, ConnectRemoteRequest};
use jade_sync::application::init_repository::{
    InitRepository, InitRepositoryError, InitRepositoryRequest,
};
use jade_sync::infrastructure::{Git2VersionControl, InMemoryCredentialStore};

use crate::world::SyncWorld;

#[given(expr = "devices {string} and {string} cloned the remote vault")]
fn two_devices_cloned_the_remote(world: &mut SyncWorld, first: String, second: String) {
    clone_remote(world, &first);
    clone_remote(world, &second);
}

#[given(expr = "device {string} cloned the remote vault")]
#[when(expr = "device {string} cloned the remote vault")]
fn a_device_cloned_the_remote(world: &mut SyncWorld, device: String) {
    clone_remote(world, &device);
}

#[given(expr = "a new local vault on device {string}")]
fn a_new_local_vault(world: &mut SyncWorld, device: String) {
    let dir = world.add_device(&device);
    initialise(dir).expect("repository is initialised");
}

#[given(expr = "a plain folder on device {string}")]
fn a_plain_folder(world: &mut SyncWorld, device: String) {
    world.add_device(&device);
}

#[given(expr = "{string} writes {string} with {string}")]
fn a_device_writes_a_note(world: &mut SyncWorld, device: String, path: String, content: String) {
    let file = world.device_dir(&device).join(path);
    fs::create_dir_all(file.parent().expect("note has a folder")).expect("folder is created");
    fs::write(file, content).expect("note is written");
}

#[given(expr = "{string} deletes {string}")]
fn a_device_deletes_a_note(world: &mut SyncWorld, device: String, path: String) {
    fs::remove_file(world.device_dir(&device).join(path)).expect("note is deleted");
}

#[given(expr = "{string} is connected to the remote vault")]
fn a_device_is_connected(world: &mut SyncWorld, device: String) {
    ConnectRemote::new(Git2VersionControl)
        .execute(ConnectRemoteRequest {
            working_directory: world.device_working_directory(&device),
            url: world.remote_url(),
        })
        .expect("remote is connected");
}

#[when(expr = "{string} is initialised again")]
fn a_device_is_initialised_again(world: &mut SyncWorld, device: String) {
    world.init_result = Some(initialise(world.device_working_directory(&device)));
}

#[then("initialising fails because it is already a repository")]
fn initialising_fails(world: &mut SyncWorld) {
    assert_eq!(
        world.init_result,
        Some(Err(InitRepositoryError::AlreadyARepository))
    );
}

#[then(expr = "{string} has {string} with {string}")]
fn a_device_has_a_note(world: &mut SyncWorld, device: String, path: String, content: String) {
    let actual = fs::read_to_string(world.device_dir(&device).join(&path))
        .unwrap_or_else(|_| panic!("{device} has no {path}"));
    assert_eq!(actual, content);
}

#[then(expr = "{string} does not have {string}")]
fn a_device_does_not_have_a_note(world: &mut SyncWorld, device: String, path: String) {
    assert!(!world.device_dir(&device).join(path).exists());
}

fn clone_remote(world: &mut SyncWorld, device: &str) {
    let dir = world.add_device(device);
    CloneRepository::new(Git2VersionControl, InMemoryCredentialStore::default())
        .execute(CloneRepositoryRequest {
            url: world.remote_url(),
            working_directory: dir,
        })
        .expect("remote is cloned");
}

fn initialise(working_directory: String) -> Result<(), InitRepositoryError> {
    InitRepository::new(Git2VersionControl).execute(InitRepositoryRequest { working_directory })
}

use cucumber::{then, when};
use jade_shared::application::use_case::UseCase;
use jade_sync::application::list_note_history::{ListNoteHistory, ListNoteHistoryRequest};
use jade_sync::application::read_note_revision::{ReadNoteRevision, ReadNoteRevisionRequest};
use jade_sync::domain::port::Revision;
use jade_sync::infrastructure::Git2VersionControl;

use crate::world::SyncWorld;

#[when(expr = "the history of {string} on {string} is requested")]
fn history_is_requested(world: &mut SyncWorld, path: String, device: String) {
    world.history = ListNoteHistory::new(Git2VersionControl)
        .execute(ListNoteHistoryRequest {
            working_directory: world.device_working_directory(&device),
            path,
            limit: 50,
        })
        .expect("history is listed");
}

#[then(expr = "the history has {int} revision(s)")]
fn history_length(world: &mut SyncWorld, expected: usize) {
    assert_eq!(world.history.len(), expected);
}

#[then(expr = "the oldest revision of {string} on {string} contains {string}")]
fn oldest_revision_contains(world: &mut SyncWorld, path: String, device: String, content: String) {
    let oldest = world.history.last().expect("history is not empty").clone();
    assert_eq!(read_revision(world, &device, path, &oldest), content);
}

#[then(expr = "the newest revision of {string} on {string} contains {string}")]
fn newest_revision_contains(world: &mut SyncWorld, path: String, device: String, content: String) {
    let newest = world.history.first().expect("history is not empty").clone();
    assert_eq!(read_revision(world, &device, path, &newest), content);
}

fn read_revision(world: &SyncWorld, device: &str, path: String, revision: &Revision) -> String {
    let bytes = ReadNoteRevision::new(Git2VersionControl)
        .execute(ReadNoteRevisionRequest {
            working_directory: world.device_working_directory(device),
            path,
            commit: revision.id.as_str().to_string(),
        })
        .expect("revision is read");
    String::from_utf8(bytes).expect("note is utf-8")
}

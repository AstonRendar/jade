use cucumber::gherkin::Step;
use cucumber::{then, when};
use jade_notes::application::find_backlinks::{FindBacklinks, FindBacklinksRequest};
use jade_notes::application::follow_link::{FollowLink, FollowLinkRequest};
use jade_notes::application::resolve_links::{ResolveLinks, ResolveLinksRequest};
use jade_notes::infrastructure::FsNoteRepository;
use jade_shared::application::use_case::UseCase;

use crate::world::NotesWorld;

#[when(expr = "the link {string} is followed")]
fn a_link_is_followed(world: &mut NotesWorld, target: String) {
    let result = FollowLink::new(FsNoteRepository).execute(FollowLinkRequest {
        vault_root: world.root(),
        target,
    });
    world.followed = world.record(result);
}

#[then(expr = "the followed note is {string}")]
fn the_followed_note_is(world: &mut NotesWorld, path: String) {
    assert_eq!(followed(world).note.as_str(), path);
}

#[then("no note was created")]
fn no_note_was_created(world: &mut NotesWorld) {
    assert!(!followed(world).created);
}

#[then("a note was created")]
fn a_note_was_created(world: &mut NotesWorld) {
    assert!(followed(world).created);
}

#[when(expr = "the backlinks of {string} are requested")]
fn backlinks_are_requested(world: &mut NotesWorld, path: String) {
    world.backlinks = FindBacklinks::new(FsNoteRepository)
        .execute(FindBacklinksRequest {
            vault_root: world.root(),
            path,
        })
        .expect("backlinks are found");
}

#[then("the backlinks are:")]
fn the_backlinks_are(world: &mut NotesWorld, step: &Step) {
    let actual: Vec<Vec<String>> = world
        .backlinks
        .iter()
        .map(|b| {
            vec![
                b.source.as_str().to_string(),
                b.line.to_string(),
                b.text.clone(),
            ]
        })
        .collect();
    assert_eq!(actual, table_rows(step));
}

#[when(expr = "the links {string} are resolved")]
fn links_are_resolved(world: &mut NotesWorld, targets: String) {
    world.resolved = ResolveLinks::new(FsNoteRepository)
        .execute(ResolveLinksRequest {
            vault_root: world.root(),
            targets: targets.split(", ").map(str::to_string).collect(),
        })
        .expect("links are resolved");
}

#[then(expr = "{string} resolves to {string}")]
fn resolves_to(world: &mut NotesWorld, target: String, path: String) {
    assert_eq!(resolution(world, &target), Some(path));
}

#[then(expr = "{string} does not resolve")]
fn does_not_resolve(world: &mut NotesWorld, target: String) {
    assert_eq!(resolution(world, &target), None);
}

pub fn table_rows(step: &Step) -> Vec<Vec<String>> {
    step.table
        .as_ref()
        .expect("step has a table")
        .rows
        .iter()
        .skip(1)
        .map(|row| row.iter().map(|cell| cell.trim().to_string()).collect())
        .collect()
}

fn followed(world: &NotesWorld) -> &jade_notes::application::follow_link::FollowedLink {
    world.followed.as_ref().expect("a link was followed")
}

fn resolution(world: &NotesWorld, target: &str) -> Option<String> {
    world
        .resolved
        .iter()
        .find(|resolved| resolved.target == target)
        .expect("target was resolved")
        .note
        .as_ref()
        .map(|note| note.as_str().to_string())
}

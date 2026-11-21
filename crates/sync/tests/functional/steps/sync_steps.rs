use cucumber::{given, then, when};

use crate::world::SyncWorld;

#[given(expr = "{string} synchronizes")]
#[when(expr = "{string} synchronizes")]
fn a_device_synchronizes(world: &mut SyncWorld, device: String) {
    let report = world.synchronize(&device);
    world.last_report = Some(report);
}

#[then("the last sync committed nothing")]
fn committed_nothing(world: &mut SyncWorld) {
    assert_eq!(last_report(world).committed, None);
}

#[then("the last sync committed the changes")]
fn committed_the_changes(world: &mut SyncWorld) {
    assert!(last_report(world).committed.is_some());
}

#[then("the last sync pushed to the remote")]
fn pushed(world: &mut SyncWorld) {
    assert!(last_report(world).pushed);
}

#[then("the last sync did not push")]
fn did_not_push(world: &mut SyncWorld) {
    assert!(!last_report(world).pushed);
}

#[then("the last sync created no conflict copies")]
fn no_conflict_copies(world: &mut SyncWorld) {
    assert!(last_report(world).conflict_copies.is_empty());
}

#[then(expr = "the last sync reported the conflict copy {string}")]
fn reported_conflict_copy(world: &mut SyncWorld, copy: String) {
    let copies: Vec<&str> = last_report(world)
        .conflict_copies
        .iter()
        .map(|conflict| conflict.copy.as_str())
        .collect();
    assert_eq!(copies, vec![copy.as_str()]);
}

fn last_report(world: &SyncWorld) -> &jade_sync::application::synchronize_vault::SyncReport {
    world.last_report.as_ref().expect("a sync ran")
}

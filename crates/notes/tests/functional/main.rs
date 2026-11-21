#![allow(clippy::expect_used)]

mod steps;
mod world;

use cucumber::World;

#[tokio::main]
async fn main() {
    world::NotesWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/features"))
        .await;
}

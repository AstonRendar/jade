#![allow(clippy::expect_used)]

mod doubles;
mod steps;
mod world;

use cucumber::World;

#[tokio::main]
async fn main() {
    world::AppWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/features"))
        .await;
}

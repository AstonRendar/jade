use cucumber::gherkin::Step;
use cucumber::{then, when};
use jade_notes::application::error::NotesError;
use jade_notes::application::search_notes::{SearchNotes, SearchNotesRequest};
use jade_notes::infrastructure::FsNoteRepository;
use jade_shared::application::use_case::UseCase;

use super::link_steps::table_rows;
use crate::world::NotesWorld;

#[when(expr = "the vault is searched for {string}")]
fn the_vault_is_searched(world: &mut NotesWorld, query: String) {
    let result = SearchNotes::new(FsNoteRepository).execute(SearchNotesRequest {
        vault_root: world.root(),
        query,
    });
    world.hits = world.record(result).unwrap_or_default();
}

#[then("the search hits are:")]
fn the_search_hits_are(world: &mut NotesWorld, step: &Step) {
    let actual: Vec<Vec<String>> = world
        .hits
        .iter()
        .map(|h| {
            vec![
                h.path.as_str().to_string(),
                h.line.to_string(),
                h.text.clone(),
            ]
        })
        .collect();
    assert_eq!(actual, table_rows(step));
}

#[then("the operation fails because the query is empty")]
fn fails_because_query_is_empty(world: &mut NotesWorld) {
    assert_eq!(world.last_error, Some(NotesError::EmptyQuery));
}

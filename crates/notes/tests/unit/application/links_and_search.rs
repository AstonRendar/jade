use jade_notes::application::error::NotesError;
use jade_notes::application::find_backlinks::{FindBacklinks, FindBacklinksRequest};
use jade_notes::application::resolve_links::{ResolveLinks, ResolveLinksRequest, ResolvedLink};
use jade_notes::application::search_notes::{SearchNotes, SearchNotesRequest};
use jade_notes::domain::service::Backlink;
use jade_shared::application::use_case::UseCase;

use crate::support::mocks::MockNoteRepository;
use crate::support::mothers::{NoteMother, VAULT};

#[test]
fn resolves_each_requested_target() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_note_paths(&mut repository, vec![NoteMother::note("plan.md", "")]);

    // Act
    let resolved = ResolveLinks::new(repository).execute(ResolveLinksRequest {
        vault_root: VAULT.to_string(),
        targets: vec!["Plan".to_string(), "Missing".to_string()],
    });

    // Assert
    assert_eq!(
        resolved,
        Ok(vec![
            ResolvedLink {
                target: "Plan".to_string(),
                note: Some(NoteMother::path("plan.md")),
            },
            ResolvedLink {
                target: "Missing".to_string(),
                note: None,
            },
        ])
    );
}

#[test]
fn finds_backlinks_of_a_note() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_notes(
        &mut repository,
        vec![
            NoteMother::note("plan.md", ""),
            NoteMother::note("daily.md", "see [[plan]]"),
        ],
    );

    // Act
    let backlinks = FindBacklinks::new(repository).execute(FindBacklinksRequest {
        vault_root: VAULT.to_string(),
        path: "plan.md".to_string(),
    });

    // Assert
    assert_eq!(
        backlinks,
        Ok(vec![Backlink {
            source: NoteMother::path("daily.md"),
            line: 1,
            text: "see [[plan]]".to_string(),
        }])
    );
}

#[test]
fn rejects_blank_search_queries() {
    // Arrange
    let repository = MockNoteRepository::new();

    // Act
    let result = SearchNotes::new(repository).execute(SearchNotesRequest {
        vault_root: VAULT.to_string(),
        query: "  ".to_string(),
    });

    // Assert
    assert_eq!(result, Err(NotesError::EmptyQuery));
}

#[test]
fn searches_every_note() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_notes(&mut repository, vec![NoteMother::note("a.md", "jade")]);

    // Act
    let hits = SearchNotes::new(repository)
        .execute(SearchNotesRequest {
            vault_root: VAULT.to_string(),
            query: "jade".to_string(),
        })
        .expect("search succeeds");

    // Assert
    assert_eq!(hits.len(), 1);
}

fn given_notes(repository: &mut MockNoteRepository, notes: Vec<jade_notes::domain::model::Note>) {
    repository.expect_notes().return_once(move |_| Ok(notes));
}

fn given_note_paths(
    repository: &mut MockNoteRepository,
    notes: Vec<jade_notes::domain::model::Note>,
) {
    let entries: Vec<_> = notes
        .into_iter()
        .map(|note| jade_notes::domain::model::VaultEntry {
            path: note.path().as_vault_path().clone(),
            kind: jade_notes::domain::model::EntryKind::Note,
        })
        .collect();
    repository.expect_list().return_once(move |_| Ok(entries));
}

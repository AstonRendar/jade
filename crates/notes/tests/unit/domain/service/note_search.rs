use jade_notes::domain::model::Note;
use jade_notes::domain::service::{NoteSearch, SearchHit};
use jade_notes::domain::value_object::SearchQuery;

use crate::support::mothers::NoteMother;

#[test]
fn finds_matching_lines_in_every_note() {
    // Arrange
    let notes = vec![
        note("a.md", "Jade notes\nnothing"),
        note("b.md", "other\n  more about JADE  "),
    ];

    // Act
    let hits = NoteSearch::default().search(&query("jade"), &notes);

    // Assert
    assert_eq!(
        hits,
        vec![
            hit("a.md", 1, "Jade notes"),
            hit("b.md", 2, "more about JADE")
        ]
    );
}

#[test]
fn matches_note_titles_even_without_content_hits() {
    // Arrange
    let notes = vec![note("projects/Jade roadmap.md", "nothing here")];

    // Act
    let hits = NoteSearch::default().search(&query("roadmap"), &notes);

    // Assert
    assert_eq!(
        hits,
        vec![hit("projects/Jade roadmap.md", 0, "Jade roadmap")]
    );
}

#[test]
fn stops_at_the_result_limit() {
    // Arrange
    let notes = vec![note("a.md", "jade\njade\njade")];

    // Act
    let hits = NoteSearch::with_limit(2).search(&query("jade"), &notes);

    // Assert
    assert_eq!(hits.len(), 2);
}

fn note(path: &str, content: &str) -> Note {
    Note::new(NoteMother::path(path), content.to_string())
}

fn query(raw: &str) -> SearchQuery {
    SearchQuery::parse(raw).expect("valid query")
}

fn hit(path: &str, line: usize, text: &str) -> SearchHit {
    SearchHit {
        path: NoteMother::path(path),
        line,
        text: text.to_string(),
    }
}

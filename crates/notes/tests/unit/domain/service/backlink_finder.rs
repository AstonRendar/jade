use jade_notes::domain::model::Note;
use jade_notes::domain::service::{Backlink, BacklinkFinder};

use crate::support::mothers::NoteMother;

#[test]
fn finds_the_lines_that_link_to_a_note() {
    // Arrange
    let notes = vec![
        note("plan.md", "# Plan"),
        note("daily.md", "intro\n- review [[Plan]] today\n- [[Other]]"),
        note("ideas.md", "see [[plan|the plan]]"),
    ];

    // Act
    let backlinks = BacklinkFinder.find(&NoteMother::path("plan.md"), &notes);

    // Assert
    assert_eq!(
        backlinks,
        vec![
            backlink("daily.md", 2, "- review [[Plan]] today"),
            backlink("ideas.md", 1, "see [[plan|the plan]]"),
        ]
    );
}

#[test]
fn ignores_links_from_the_note_to_itself() {
    // Arrange
    let notes = vec![note("plan.md", "see [[Plan#Goals]]")];

    // Act
    let backlinks = BacklinkFinder.find(&NoteMother::path("plan.md"), &notes);

    // Assert
    assert_eq!(backlinks, Vec::new());
}

#[test]
fn ignores_links_that_resolve_to_another_note_with_the_same_name() {
    // Arrange
    let notes = vec![
        note("plan.md", ""),
        note("work/plan.md", ""),
        note("daily.md", "[[work/plan]]"),
    ];

    // Act
    let backlinks = BacklinkFinder.find(&NoteMother::path("plan.md"), &notes);

    // Assert
    assert_eq!(backlinks, Vec::new());
}

#[test]
fn reports_each_line_once_even_with_several_links() {
    // Arrange
    let notes = vec![
        note("plan.md", ""),
        note("daily.md", "[[plan]] and [[Plan]]"),
    ];

    // Act
    let backlinks = BacklinkFinder.find(&NoteMother::path("plan.md"), &notes);

    // Assert
    assert_eq!(
        backlinks,
        vec![backlink("daily.md", 1, "[[plan]] and [[Plan]]")]
    );
}

fn note(path: &str, content: &str) -> Note {
    Note::new(NoteMother::path(path), content.to_string())
}

fn backlink(source: &str, line: usize, text: &str) -> Backlink {
    Backlink {
        source: NoteMother::path(source),
        line,
        text: text.to_string(),
    }
}

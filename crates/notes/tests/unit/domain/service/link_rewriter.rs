use jade_notes::domain::service::{LinkRewriter, NoteMove};

use crate::support::mothers::NoteMother;

#[test]
fn points_links_to_the_new_name() {
    // Arrange
    let moves = [note_move("plan.md", "Roadmap.md")];

    // Act
    let rewritten = rewrite("See [[Plan]] today.", &moves, &["plan.md", "ideas.md"]);

    // Assert
    assert_eq!(rewritten.as_deref(), Some("See [[Roadmap]] today."));
}

#[test]
fn keeps_headings_aliases_and_embeds() {
    // Arrange
    let moves = [note_move("plan.md", "Roadmap.md")];

    // Act
    let rewritten = rewrite("![[Plan#Goals|our goals]]", &moves, &["plan.md"]);

    // Assert
    assert_eq!(rewritten.as_deref(), Some("![[Roadmap#Goals|our goals]]"));
}

#[test]
fn keeps_path_style_links_as_paths() {
    // Arrange
    let moves = [note_move("projects/plan.md", "archive/plan.md")];

    // Act
    let rewritten = rewrite("[[projects/plan]]", &moves, &["projects/plan.md"]);

    // Assert
    assert_eq!(rewritten.as_deref(), Some("[[archive/plan]]"));
}

#[test]
fn leaves_links_that_still_resolve_to_the_moved_note() {
    // Arrange
    let moves = [note_move("plan.md", "work/plan.md")];

    // Act
    let rewritten = rewrite("[[plan]]", &moves, &["plan.md", "other/deep/plan.md"]);

    // Assert
    assert_eq!(rewritten, None);
}

#[test]
fn uses_the_path_when_the_new_name_would_be_ambiguous() {
    // Arrange
    let moves = [note_move("plan.md", "a/b/plan.md")];

    // Act
    let rewritten = rewrite("[[plan]]", &moves, &["plan.md", "x/plan.md"]);

    // Assert
    assert_eq!(rewritten.as_deref(), Some("[[a/b/plan]]"));
}

#[test]
fn leaves_other_links_and_code_untouched() {
    // Arrange
    let moves = [note_move("plan.md", "Roadmap.md")];
    let content = "[[Ideas]] `[[Plan]]`\n```\n[[Plan]]\n```\n[[Plan]]";

    // Act
    let rewritten = rewrite(content, &moves, &["plan.md", "ideas.md"]);

    // Assert
    assert_eq!(
        rewritten.as_deref(),
        Some("[[Ideas]] `[[Plan]]`\n```\n[[Plan]]\n```\n[[Roadmap]]")
    );
}

#[test]
fn rewrites_several_links_on_a_line_with_multibyte_text() {
    // Arrange
    let moves = [note_move("plan.md", "Planificación.md")];

    // Act
    let rewritten = rewrite("ñ [[Plan]] y [[plan|el plan]]", &moves, &["plan.md"]);

    // Assert
    assert_eq!(
        rewritten.as_deref(),
        Some("ñ [[Planificación]] y [[Planificación|el plan]]")
    );
}

fn note_move(from: &str, to: &str) -> NoteMove {
    NoteMove {
        from: NoteMother::path(from),
        to: NoteMother::path(to),
    }
}

fn rewrite(content: &str, moves: &[NoteMove], before: &[&str]) -> Option<String> {
    LinkRewriter.rewrite(content, moves, &NoteMother::paths(before))
}

use jade_notes::domain::service::LinkResolver;

use crate::support::mothers::NoteMother;

#[test]
fn resolves_an_exact_vault_path() {
    // Arrange
    let notes = NoteMother::paths(&["plan.md", "projects/plan.md"]);

    // Act
    let resolved = LinkResolver.resolve("projects/plan", &notes);

    // Assert
    assert_eq!(resolved, Some(NoteMother::path("projects/plan.md")));
}

#[test]
fn resolves_a_note_name_ignoring_case() {
    // Arrange
    let notes = NoteMother::paths(&["projects/Meeting Notes.md"]);

    // Act
    let resolved = LinkResolver.resolve("meeting notes", &notes);

    // Assert
    assert_eq!(
        resolved,
        Some(NoteMother::path("projects/Meeting Notes.md"))
    );
}

#[test]
fn prefers_the_shortest_path_when_a_name_is_ambiguous() {
    // Arrange
    let notes = NoteMother::paths(&["archive/2025/plan.md", "work/plan.md", "zeta/deep/plan.md"]);

    // Act
    let resolved = LinkResolver.resolve("plan", &notes);

    // Assert
    assert_eq!(resolved, Some(NoteMother::path("work/plan.md")));
}

#[test]
fn accepts_targets_written_with_the_extension() {
    // Arrange
    let notes = NoteMother::paths(&["plan.md"]);

    // Act
    let resolved = LinkResolver.resolve("plan.md", &notes);

    // Assert
    assert_eq!(resolved, Some(NoteMother::path("plan.md")));
}

#[test]
fn resolves_partial_folder_paths() {
    // Arrange
    let notes = NoteMother::paths(&["work/projects/plan.md", "home/plan.md"]);

    // Act
    let resolved = LinkResolver.resolve("projects/plan", &notes);

    // Assert
    assert_eq!(resolved, Some(NoteMother::path("work/projects/plan.md")));
}

#[test]
fn resolves_nothing_when_no_note_matches() {
    // Arrange
    let notes = NoteMother::paths(&["plan.md"]);

    // Act
    let resolved = LinkResolver.resolve("missing", &notes);

    // Assert
    assert_eq!(resolved, None);
}

#[test]
fn matches_decomposed_targets_with_composed_names() {
    // Arrange
    let notes = NoteMother::paths(&["Caf\u{e9}.md"]);

    // Act
    let resolved = LinkResolver.resolve("Cafe\u{301}", &notes);

    // Assert
    assert_eq!(resolved, Some(NoteMother::path("Caf\u{e9}.md")));
}

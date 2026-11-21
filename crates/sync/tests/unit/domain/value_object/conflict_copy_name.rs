use jade_sync::domain::value_object::{ConflictCopyName, TrackedPath};

use crate::support::mothers::{TimestampMother, TrackedPathMother};

#[test]
fn inserts_the_conflict_suffix_before_the_extension() {
    // Arrange
    let original = TrackedPathMother::of("ideas.md");

    // Act
    let copy = ConflictCopyName::for_conflict(&original, &TimestampMother::any(), nothing_taken);

    // Assert
    assert_eq!(copy.path().as_str(), "ideas (conflict 2026-11-21 2245).md");
}

#[test]
fn keeps_the_copy_in_the_same_folder() {
    // Arrange
    let original = TrackedPathMother::of("projects/jade/plan.md");

    // Act
    let copy = ConflictCopyName::for_conflict(&original, &TimestampMother::any(), nothing_taken);

    // Assert
    assert_eq!(
        copy.path().as_str(),
        "projects/jade/plan (conflict 2026-11-21 2245).md"
    );
}

#[test]
fn appends_the_suffix_to_files_without_extension_or_dotfiles() {
    // Arrange
    let readme = TrackedPathMother::of("README");
    let dotfile = TrackedPathMother::of(".gitignore");

    // Act
    let readme_copy =
        ConflictCopyName::for_conflict(&readme, &TimestampMother::any(), nothing_taken);
    let dotfile_copy =
        ConflictCopyName::for_conflict(&dotfile, &TimestampMother::any(), nothing_taken);

    // Assert
    assert_eq!(
        readme_copy.path().as_str(),
        "README (conflict 2026-11-21 2245)"
    );
    assert_eq!(
        dotfile_copy.path().as_str(),
        ".gitignore (conflict 2026-11-21 2245)"
    );
}

#[test]
fn only_treats_the_last_dot_as_the_extension() {
    // Arrange
    let original = TrackedPathMother::of("archive.tar.gz");

    // Act
    let copy = ConflictCopyName::for_conflict(&original, &TimestampMother::any(), nothing_taken);

    // Assert
    assert_eq!(
        copy.path().as_str(),
        "archive.tar (conflict 2026-11-21 2245).gz"
    );
}

#[test]
fn pads_date_and_time_components() {
    // Arrange
    let original = TrackedPathMother::of("ideas.md");
    let early = TimestampMother::at(2026, 1, 5, 9, 7);

    // Act
    let copy = ConflictCopyName::for_conflict(&original, &early, nothing_taken);

    // Assert
    assert_eq!(copy.path().as_str(), "ideas (conflict 2026-01-05 0907).md");
}

#[test]
fn numbers_the_copy_when_the_name_is_already_taken() {
    // Arrange
    let original = TrackedPathMother::of("ideas.md");
    let taken = [
        "ideas (conflict 2026-11-21 2245).md",
        "ideas (conflict 2026-11-21 2245 2).md",
    ];

    // Act
    let copy = ConflictCopyName::for_conflict(&original, &TimestampMother::any(), |path| {
        taken.contains(&path.as_str())
    });

    // Assert
    assert_eq!(
        copy.path().as_str(),
        "ideas (conflict 2026-11-21 2245 3).md"
    );
}

fn nothing_taken(_: &TrackedPath) -> bool {
    false
}

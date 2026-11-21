use jade_notes::domain::value_object::{NotePath, VaultPath};

#[test]
fn accepts_markdown_files_in_any_case() {
    // Arrange
    let raws = ["ideas.md", "projects/Plan.MD"];

    // Act
    let accepted: Vec<bool> = raws
        .iter()
        .map(|raw| NotePath::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, vec![true, true]);
}

#[test]
fn rejects_files_that_are_not_markdown() {
    // Arrange
    let raws = ["diagram.png", "notes", ".md", "folder/.md"];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| NotePath::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

#[test]
fn takes_its_title_from_the_file_name() {
    // Arrange
    let path = NotePath::parse("projects/Meeting notes.md").expect("valid");

    // Act
    let title = path.title();

    // Assert
    assert_eq!(title, "Meeting notes");
}

#[test]
fn builds_a_note_path_from_a_title_inside_a_folder() {
    // Arrange
    let folder = VaultPath::parse("projects").expect("valid");

    // Act
    let path = NotePath::from_title(Some(&folder), "Plan").expect("valid");

    // Assert
    assert_eq!(path.as_str(), "projects/Plan.md");
}

#[test]
fn builds_a_note_path_from_a_title_at_the_root() {
    // Arrange
    // Act
    let path = NotePath::from_title(None, "  Plan  ").expect("valid");

    // Assert
    assert_eq!(path.as_str(), "Plan.md");
}

#[test]
fn rejects_titles_that_would_leave_the_folder() {
    // Arrange
    let titles = ["", "  ", "a/b", "..", "."];

    // Act
    let accepted: Vec<&str> = titles
        .into_iter()
        .filter(|title| NotePath::from_title(None, title).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

#[test]
fn exposes_its_generic_vault_path() {
    // Arrange
    let path = NotePath::parse("projects/plan.md").expect("valid");

    // Act
    let vault_path = path.as_vault_path();

    // Assert
    assert_eq!(vault_path.as_str(), "projects/plan.md");
}

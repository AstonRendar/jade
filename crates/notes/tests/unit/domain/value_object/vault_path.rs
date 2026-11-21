use jade_notes::domain::value_object::VaultPath;

#[test]
fn accepts_relative_slash_separated_paths() {
    // Arrange
    let raw = "projects/jade/diagram.png";

    // Act
    let path = VaultPath::parse(raw).expect("valid");

    // Assert
    assert_eq!(path.as_str(), raw);
}

#[test]
fn normalises_to_composed_unicode() {
    // Arrange
    let decomposed = "Cafe\u{301}.md";

    // Act
    let path = VaultPath::parse(decomposed).expect("valid");

    // Assert
    assert_eq!(path.as_str(), "Caf\u{e9}.md");
}

#[test]
fn rejects_paths_that_escape_the_vault_or_are_not_normalised() {
    // Arrange
    let raws = [
        "",
        "/absolute.md",
        "../outside.md",
        "a/../b.md",
        "./here.md",
        "a//b.md",
        "a\\b.md",
        "folder/",
        "line\nbreak.md",
    ];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| VaultPath::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

#[test]
fn exposes_its_name_and_parent() {
    // Arrange
    let path = VaultPath::parse("projects/jade/plan.md").expect("valid");

    // Act
    let name = path.file_name();
    let parent = path.parent();

    // Assert
    assert_eq!(name, "plan.md");
    assert_eq!(
        parent.map(|p| p.as_str().to_string()),
        Some("projects/jade".to_string())
    );
}

#[test]
fn has_no_parent_at_the_vault_root() {
    // Arrange
    let path = VaultPath::parse("plan.md").expect("valid");

    // Act
    let parent = path.parent();

    // Assert
    assert!(parent.is_none());
}

#[test]
fn is_hidden_when_any_segment_starts_with_a_dot() {
    // Arrange
    let hidden = [".obsidian/app.json", ".git/config", "notes/.draft.md"];
    let visible = "notes/draft.md";

    // Act
    let hidden_flags: Vec<bool> = hidden
        .iter()
        .map(|raw| VaultPath::parse(raw).expect("valid").is_hidden())
        .collect();
    let visible_flag = VaultPath::parse(visible).expect("valid").is_hidden();

    // Assert
    assert_eq!(hidden_flags, vec![true, true, true]);
    assert!(!visible_flag);
}

#[test]
fn joins_a_child_name() {
    // Arrange
    let folder = VaultPath::parse("projects").expect("valid");

    // Act
    let child = folder.join("plan.md").expect("valid child");

    // Assert
    assert_eq!(child.as_str(), "projects/plan.md");
}

#[test]
fn rejects_child_names_with_separators() {
    // Arrange
    let folder = VaultPath::parse("projects").expect("valid");

    // Act
    let result = folder.join("a/b.md");

    // Assert
    assert!(result.is_err());
}

#[test]
fn knows_whether_it_is_inside_a_folder() {
    // Arrange
    let folder = VaultPath::parse("projects").expect("valid");
    let inside = VaultPath::parse("projects/jade/plan.md").expect("valid");
    let sibling = VaultPath::parse("projects-old/plan.md").expect("valid");

    // Act
    let results = (
        inside.is_within(&folder),
        sibling.is_within(&folder),
        folder.is_within(&folder),
    );

    // Assert
    assert_eq!(results, (true, false, true));
}

use jade_notes::domain::service::Frontmatter;

#[test]
fn splits_the_yaml_block_from_the_body() {
    // Arrange
    let content = "---\ntype: character\n---\n# Title\n";

    // Act
    let frontmatter = Frontmatter::split(content);

    // Assert
    assert_eq!(frontmatter.yaml, Some("type: character\n"));
    assert_eq!(frontmatter.body, "# Title\n");
}

#[test]
fn accepts_windows_line_endings_and_an_empty_block() {
    // Arrange
    let content = "---\r\n---\r\nbody";

    // Act
    let frontmatter = Frontmatter::split(content);

    // Assert
    assert_eq!(frontmatter.yaml, Some(""));
    assert_eq!(frontmatter.body, "body");
}

#[test]
fn treats_notes_without_a_closed_block_as_body_only() {
    // Arrange
    let contents = ["# Title\n---\n", "---\ntype: open\n"];

    // Act
    let splits: Vec<_> = contents
        .iter()
        .map(|content| Frontmatter::split(content))
        .collect();

    // Assert
    assert_eq!(splits[0].yaml, None);
    assert_eq!(splits[0].body, contents[0]);
    assert_eq!(splits[1].yaml, None);
    assert_eq!(splits[1].body, contents[1]);
}

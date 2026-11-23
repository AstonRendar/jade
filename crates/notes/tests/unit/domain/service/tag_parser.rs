use jade_notes::domain::service::TagParser;

#[test]
fn finds_tags_and_nested_tags_in_the_text() {
    // Arrange
    let body = "#idea about #project/jade, see #año-2026\n";

    // Act
    let tags = TagParser.parse(body);

    // Assert
    assert_eq!(tags, vec!["idea", "project/jade", "año-2026"]);
}

#[test]
fn ignores_headings_numbers_anchors_and_code() {
    // Arrange
    let body = "# Heading\nissue #123 and [[Note#Section]]\n`#code` here\n```\n#fenced\n```\n";

    // Act
    let tags = TagParser.parse(body);

    // Assert
    assert!(tags.is_empty(), "{tags:?}");
}

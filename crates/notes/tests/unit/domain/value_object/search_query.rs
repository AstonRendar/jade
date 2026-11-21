use jade_notes::domain::value_object::SearchQuery;

#[test]
fn matches_lines_ignoring_case() {
    // Arrange
    let query = SearchQuery::parse("JADE").expect("valid");

    // Act
    let matches = (query.matches("Notes about jade"), query.matches("Obsidian"));

    // Assert
    assert_eq!(matches, (true, false));
}

#[test]
fn trims_the_query() {
    // Arrange
    // Act
    let query = SearchQuery::parse("  jade  ").expect("valid");

    // Assert
    assert_eq!(query.as_str(), "jade");
}

#[test]
fn rejects_blank_queries() {
    // Arrange
    // Act
    let result = SearchQuery::parse("   ");

    // Assert
    assert!(result.is_err());
}

#[test]
fn matches_composed_and_decomposed_text_alike() {
    // Arrange
    let query = SearchQuery::parse("cafe\u{301}").expect("valid");

    // Act
    let matched = query.matches("Un CAF\u{c9} con leche");

    // Assert
    assert!(matched);
}

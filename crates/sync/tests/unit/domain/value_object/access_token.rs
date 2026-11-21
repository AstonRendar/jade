use jade_sync::domain::value_object::AccessToken;

#[test]
fn exposes_the_secret_on_demand() {
    // Arrange
    let raw = "ghp_example_secret";

    // Act
    let token = AccessToken::new(raw).expect("valid");

    // Assert
    assert_eq!(token.expose(), "ghp_example_secret");
}

#[test]
fn never_shows_the_secret_when_debug_printed() {
    // Arrange
    let token = AccessToken::new("ghp_example_secret").expect("valid");

    // Act
    let printed = format!("{token:?}");

    // Assert
    assert!(!printed.contains("ghp_example_secret"));
}

#[test]
fn rejects_blank_tokens_or_tokens_with_whitespace() {
    // Arrange
    let raws = ["", "   ", "ghp abc", "ghp\nabc"];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| AccessToken::new(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

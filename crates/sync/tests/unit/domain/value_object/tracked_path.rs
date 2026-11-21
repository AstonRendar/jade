use jade_sync::domain::value_object::TrackedPath;

#[test]
fn accepts_relative_slash_separated_paths() {
    // Arrange
    let raw = "projects/jade/plan.md";

    // Act
    let path = TrackedPath::parse(raw).expect("valid");

    // Assert
    assert_eq!(path.as_str(), raw);
}

#[test]
fn rejects_paths_that_escape_or_are_not_normalised() {
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
        "a/./b.md",
    ];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| TrackedPath::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

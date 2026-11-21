use jade_sync::domain::value_object::BranchName;

#[test]
fn accepts_conventional_branch_names() {
    // Arrange
    let names = ["main", "feature/notes", "release-1.2", "user_name"];

    // Act
    let parsed: Vec<String> = names
        .iter()
        .map(|name| BranchName::parse(name).expect("valid").as_str().to_string())
        .collect();

    // Assert
    assert_eq!(parsed, names);
}

#[test]
fn rejects_names_git_does_not_allow() {
    // Arrange
    let names = [
        "",
        " ",
        "has space",
        "-leading",
        "/leading",
        "trailing/",
        "double..dot",
        "ends.lock",
        "a~b",
        "a^b",
        "a:b",
        "a?b",
        "a*b",
        "a[b",
        "a\\b",
        "a@{b",
        "tab\there",
        "double//slash",
        "ends.",
    ];

    // Act
    let accepted: Vec<&str> = names
        .into_iter()
        .filter(|name| BranchName::parse(name).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

use jade_sync::domain::value_object::WorkingDirectory;

#[test]
fn accepts_absolute_paths() {
    // Arrange
    let raw = "/vaults/notes";

    // Act
    let dir = WorkingDirectory::parse(raw).expect("valid");

    // Assert
    assert_eq!(dir.as_path().to_str(), Some(raw));
}

#[test]
fn rejects_relative_or_empty_paths() {
    // Arrange
    let raws = ["", "vaults/notes", "./notes"];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| WorkingDirectory::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

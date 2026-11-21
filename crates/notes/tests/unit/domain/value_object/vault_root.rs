use jade_notes::domain::value_object::VaultRoot;

#[test]
fn accepts_absolute_folders_and_names_the_vault_after_them() {
    // Arrange
    let raw = "/Users/example/Notes";

    // Act
    let root = VaultRoot::parse(raw).expect("valid");

    // Assert
    assert_eq!(root.as_path().to_str(), Some(raw));
    assert_eq!(root.name(), "Notes");
}

#[test]
fn rejects_relative_folders() {
    // Arrange
    let raws = ["", "Notes", "./Notes"];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| VaultRoot::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

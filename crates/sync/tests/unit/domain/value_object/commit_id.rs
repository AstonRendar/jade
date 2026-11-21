use jade_sync::domain::value_object::CommitId;

#[test]
fn accepts_sha1_ids_and_normalises_them_to_lowercase() {
    // Arrange
    let raw = "ABCDEF0123456789ABCDEF0123456789ABCDEF01";

    // Act
    let id = CommitId::parse(raw).expect("valid");

    // Assert
    assert_eq!(id.as_str(), "abcdef0123456789abcdef0123456789abcdef01");
}

#[test]
fn accepts_sha256_ids() {
    // Arrange
    let raw = "a".repeat(64);

    // Act
    let result = CommitId::parse(&raw);

    // Assert
    assert!(result.is_ok());
}

#[test]
fn rejects_ids_of_other_lengths_or_non_hex() {
    // Arrange
    let raws = ["", "abc123", &"g".repeat(40), &"a".repeat(41)];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| CommitId::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

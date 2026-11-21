use jade_sync::domain::value_object::DeviceName;

#[test]
fn trims_surrounding_whitespace() {
    // Arrange
    let raw = "  laptop  ";

    // Act
    let name = DeviceName::parse(raw).expect("valid");

    // Assert
    assert_eq!(name.as_str(), "laptop");
}

#[test]
fn rejects_blank_names() {
    // Arrange
    let raw = "   ";

    // Act
    let result = DeviceName::parse(raw);

    // Assert
    assert!(result.is_err());
}

#[test]
fn rejects_control_characters() {
    // Arrange
    let raw = "lap\ntop";

    // Act
    let result = DeviceName::parse(raw);

    // Assert
    assert!(result.is_err());
}

#[test]
fn rejects_names_longer_than_64_characters() {
    // Arrange
    let longest = "a".repeat(64);
    let too_long = "a".repeat(65);

    // Act
    let accepted = DeviceName::parse(&longest);
    let rejected = DeviceName::parse(&too_long);

    // Assert
    assert!(accepted.is_ok());
    assert!(rejected.is_err());
}

#[test]
fn has_a_generic_name_for_unknown_devices() {
    // Arrange
    // Act
    let name = DeviceName::unknown();

    // Assert
    assert_eq!(name.as_str(), "Unknown device");
}

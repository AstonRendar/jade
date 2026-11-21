use jade_sync::domain::value_object::{CommitMessage, DeviceName};

#[test]
fn keeps_a_meaningful_message() {
    // Arrange
    let text = "Add meeting notes";

    // Act
    let message = CommitMessage::new(text).expect("valid");

    // Assert
    assert_eq!(message.as_str(), "Add meeting notes");
}

#[test]
fn rejects_a_blank_message() {
    // Arrange
    let text = "  \n ";

    // Act
    let result = CommitMessage::new(text);

    // Assert
    assert!(result.is_err());
}

#[test]
fn names_sync_commits_after_the_device() {
    // Arrange
    let device = DeviceName::parse("laptop").expect("valid");

    // Act
    let message = CommitMessage::sync_from(&device);

    // Assert
    assert_eq!(message.as_str(), "Sync from laptop");
}

#[test]
fn has_a_fixed_english_message_for_merges() {
    // Arrange
    // Act
    let message = CommitMessage::merge_remote_changes();

    // Assert
    assert_eq!(message.as_str(), "Merge remote changes");
}

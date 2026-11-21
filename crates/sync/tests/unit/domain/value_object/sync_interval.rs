use jade_sync::domain::value_object::SyncInterval;

#[test]
fn defaults_to_five_minutes() {
    // Arrange
    // Act
    let interval = SyncInterval::default();

    // Assert
    assert_eq!(interval.minutes(), 5);
    assert!(interval.is_enabled());
}

#[test]
fn zero_minutes_disables_automatic_sync() {
    // Arrange
    // Act
    let interval = SyncInterval::from_minutes(0).expect("valid");

    // Assert
    assert!(!interval.is_enabled());
}

#[test]
fn rejects_intervals_longer_than_a_day() {
    // Arrange
    // Act
    let one_day = SyncInterval::from_minutes(1440);
    let longer = SyncInterval::from_minutes(1441);

    // Assert
    assert!(one_day.is_ok());
    assert!(longer.is_err());
}

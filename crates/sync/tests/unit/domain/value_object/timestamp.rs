use jade_sync::domain::value_object::Timestamp;

#[test]
fn accepts_a_valid_date_and_time() {
    // Arrange
    // Act
    let timestamp = Timestamp::new(2026, 11, 21, 22, 45).expect("valid");

    // Assert
    assert_eq!(timestamp.year(), 2026);
    assert_eq!(timestamp.month(), 11);
    assert_eq!(timestamp.day(), 21);
    assert_eq!(timestamp.hour(), 22);
    assert_eq!(timestamp.minute(), 45);
}

#[test]
fn accepts_the_29th_of_february_only_in_leap_years() {
    // Arrange
    // Act
    let leap = Timestamp::new(2028, 2, 29, 0, 0);
    let common = Timestamp::new(2026, 2, 29, 0, 0);
    let century = Timestamp::new(2100, 2, 29, 0, 0);
    let four_hundred = Timestamp::new(2000, 2, 29, 0, 0);

    // Assert
    assert!(leap.is_ok());
    assert!(common.is_err());
    assert!(century.is_err());
    assert!(four_hundred.is_ok());
}

#[test]
fn rejects_out_of_range_components() {
    // Arrange
    let candidates = [
        (2026, 0, 1, 0, 0),
        (2026, 13, 1, 0, 0),
        (2026, 4, 31, 0, 0),
        (2026, 1, 0, 0, 0),
        (2026, 1, 1, 24, 0),
        (2026, 1, 1, 0, 60),
    ];

    // Act
    let accepted: Vec<_> = candidates
        .into_iter()
        .filter(|(y, mo, d, h, mi)| Timestamp::new(*y, *mo, *d, *h, *mi).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::new());
}

#[test]
fn offers_the_unix_epoch_as_a_known_valid_instant() {
    // Arrange
    // Act
    let epoch = Timestamp::unix_epoch();

    // Assert
    assert_eq!((epoch.year(), epoch.month(), epoch.day()), (1970, 1, 1));
}

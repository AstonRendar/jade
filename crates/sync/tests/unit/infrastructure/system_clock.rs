use chrono::NaiveDate;
use jade_sync::infrastructure::SystemClock;

#[test]
fn keeps_the_local_date_and_minute() {
    // Arrange
    let local = NaiveDate::from_ymd_opt(2026, 11, 21)
        .and_then(|date| date.and_hms_opt(22, 45, 59))
        .expect("valid date");

    // Act
    let timestamp = SystemClock::timestamp_of(local);

    // Assert
    assert_eq!(
        (
            timestamp.year(),
            timestamp.month(),
            timestamp.day(),
            timestamp.hour(),
            timestamp.minute()
        ),
        (2026, 11, 21, 22, 45)
    );
}

use jade_sync::domain::service::SyncRetryPolicy;

#[test]
fn allows_three_attempts_by_default() {
    // Arrange
    let policy = SyncRetryPolicy::default();

    // Act
    let after_each_attempt: Vec<bool> = (1..=3)
        .map(|attempts| policy.allows_another_attempt(attempts))
        .collect();

    // Assert
    assert_eq!(after_each_attempt, vec![true, true, false]);
}

#[test]
fn can_be_configured_with_another_maximum() {
    // Arrange
    let policy = SyncRetryPolicy::with_max_attempts(1).expect("valid");

    // Act
    let allowed = policy.allows_another_attempt(1);

    // Assert
    assert!(!allowed);
}

#[test]
fn requires_at_least_one_attempt() {
    // Arrange
    // Act
    let result = SyncRetryPolicy::with_max_attempts(0);

    // Assert
    assert!(result.is_err());
}

use jade_preferences::domain::model::Preferences;
use jade_preferences::domain::value_object::{AutoSyncMinutes, Locale, Theme};

#[test]
fn follows_the_system_by_default() {
    // Arrange
    // Act
    let preferences = Preferences::default();

    // Assert
    assert_eq!(preferences.locale(), None);
    assert_eq!(preferences.theme(), Theme::System);
    assert_eq!(preferences.auto_sync_minutes(), AutoSyncMinutes::default());
    assert_eq!(preferences.last_vault(), None);
}

#[test]
fn changes_appearance_and_sync_settings() {
    // Arrange
    let preferences = Preferences::default();

    // Act
    let changed = preferences.with_settings(
        Some(Locale::parse("ca").expect("valid")),
        Theme::Dark,
        AutoSyncMinutes::new(15).expect("valid"),
    );

    // Assert
    assert_eq!(changed.locale().map(|l| l.as_str()), Some("ca"));
    assert_eq!(changed.theme(), Theme::Dark);
    assert_eq!(changed.auto_sync_minutes().value(), 15);
}

#[test]
fn the_last_vault_is_the_most_recently_remembered() {
    // Arrange
    let preferences = Preferences::default()
        .remember_vault("/vaults/a")
        .and_then(|p| p.remember_vault("/vaults/b"))
        .expect("valid");

    // Act
    let last = preferences.last_vault();

    // Assert
    assert_eq!(last, Some("/vaults/b"));
}

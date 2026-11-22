use jade_preferences::domain::value_object::{AutoSyncMinutes, Locale, RecentVaults, Theme};

#[test]
fn parses_the_four_supported_locales() {
    // Arrange
    let raws = ["en", "es", "zh-CN", "ca"];

    // Act
    let parsed: Vec<&str> = raws
        .iter()
        .map(|raw| Locale::parse(raw).expect("valid").as_str())
        .collect();

    // Assert
    assert_eq!(parsed, raws);
}

#[test]
fn rejects_unsupported_locales() {
    // Arrange
    let raws = ["", "fr", "zh-TW", "EN"];

    // Act
    let accepted: Vec<&str> = raws
        .into_iter()
        .filter(|raw| Locale::parse(raw).is_ok())
        .collect();

    // Assert
    assert_eq!(accepted, Vec::<&str>::new());
}

#[test]
fn parses_themes() {
    // Arrange
    let raws = ["system", "light", "dark"];

    // Act
    let parsed: Vec<&str> = raws
        .iter()
        .map(|raw| Theme::parse(raw).expect("valid").as_str())
        .collect();

    // Assert
    assert_eq!(parsed, raws);
    assert!(Theme::parse("blue").is_err());
}

#[test]
fn limits_automatic_sync_to_a_day() {
    // Arrange
    // Act
    let off = AutoSyncMinutes::new(0).expect("valid");
    let too_long = AutoSyncMinutes::new(1441);

    // Assert
    assert_eq!(off.value(), 0);
    assert_eq!(AutoSyncMinutes::default().value(), 5);
    assert!(too_long.is_err());
}

#[test]
fn remembers_vaults_most_recent_first_without_duplicates() {
    // Arrange
    let recent = RecentVaults::default()
        .remember("/vaults/a")
        .and_then(|r| r.remember("/vaults/b"))
        .and_then(|r| r.remember("/vaults/a"))
        .expect("valid paths");

    // Act
    let paths = recent.paths();

    // Assert
    assert_eq!(paths, ["/vaults/a", "/vaults/b"]);
}

#[test]
fn keeps_at_most_eight_vaults() {
    // Arrange
    let mut recent = RecentVaults::default();
    for index in 0..10 {
        recent = recent.remember(&format!("/vaults/{index}")).expect("valid");
    }

    // Act
    let paths = recent.paths();

    // Assert
    assert_eq!(paths.len(), 8);
    assert_eq!(paths[0], "/vaults/9");
}

#[test]
fn forgets_a_vault() {
    // Arrange
    let recent = RecentVaults::default()
        .remember("/vaults/a")
        .and_then(|r| r.remember("/vaults/b"))
        .expect("valid");

    // Act
    let recent = recent.forget("/vaults/a");

    // Assert
    assert_eq!(recent.paths(), ["/vaults/b"]);
}

#[test]
fn only_remembers_absolute_paths() {
    // Arrange
    // Act
    let result = RecentVaults::default().remember("relative/vault");

    // Assert
    assert!(result.is_err());
}

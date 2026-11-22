use jade_preferences::application::error::PreferencesError;
use jade_preferences::application::forget_vault::{ForgetVault, ForgetVaultRequest};
use jade_preferences::application::get_preferences::GetPreferences;
use jade_preferences::application::remember_vault::{RememberVault, RememberVaultRequest};
use jade_preferences::application::update_preferences::{
    UpdatePreferences, UpdatePreferencesRequest,
};
use jade_preferences::domain::model::Preferences;
use jade_preferences::domain::port::PreferencesStorageError;
use jade_preferences::domain::value_object::Theme;
use jade_shared::application::use_case::UseCase;

use crate::support::MockPreferencesRepository;

#[test]
fn returns_the_stored_preferences() {
    // Arrange
    let mut repository = MockPreferencesRepository::new();
    given_stored(&mut repository, Preferences::default());

    // Act
    let preferences = GetPreferences::new(repository).execute(());

    // Assert
    assert_eq!(preferences, Ok(Preferences::default()));
}

#[test]
fn updates_and_saves_the_settings() {
    // Arrange
    let mut repository = MockPreferencesRepository::new();
    given_stored(&mut repository, Preferences::default());
    expect_saved_with_theme(&mut repository, Theme::Dark);

    // Act
    let updated = UpdatePreferences::new(repository).execute(update_request(Some("es"), "dark"));

    // Assert
    let updated = updated.expect("update succeeds");
    assert_eq!(updated.locale().map(|l| l.as_str()), Some("es"));
    assert_eq!(updated.theme(), Theme::Dark);
}

#[test]
fn rejects_an_unsupported_language_without_saving() {
    // Arrange
    let mut repository = MockPreferencesRepository::new();
    given_stored(&mut repository, Preferences::default());

    // Act
    let result = UpdatePreferences::new(repository).execute(update_request(Some("fr"), "dark"));

    // Assert
    assert_eq!(
        result,
        Err(PreferencesError::InvalidLocale("fr".to_string()))
    );
}

#[test]
fn remembers_and_forgets_vaults() {
    // Arrange
    let mut repository = MockPreferencesRepository::new();
    given_stored(&mut repository, Preferences::default());
    repository.expect_save().times(2).returning(|_| Ok(()));
    let remember = RememberVault::new(&repository);
    let forget = ForgetVault::new(&repository);

    // Act
    let remembered = remember
        .execute(RememberVaultRequest {
            path: "/vaults/a".to_string(),
        })
        .expect("remembered");
    let forgotten = forget
        .execute(ForgetVaultRequest {
            path: "/vaults/a".to_string(),
        })
        .expect("forgotten");

    // Assert
    assert_eq!(remembered.last_vault(), Some("/vaults/a"));
    assert_eq!(forgotten.last_vault(), None);
}

#[test]
fn reports_storage_failures() {
    // Arrange
    let mut repository = MockPreferencesRepository::new();
    repository
        .expect_load()
        .returning(|| Err(PreferencesStorageError("unreadable".to_string())));

    // Act
    let result = GetPreferences::new(repository).execute(());

    // Assert
    assert_eq!(
        result,
        Err(PreferencesError::Storage("unreadable".to_string()))
    );
}

fn update_request(locale: Option<&str>, theme: &str) -> UpdatePreferencesRequest {
    UpdatePreferencesRequest {
        locale: locale.map(str::to_string),
        theme: theme.to_string(),
        auto_sync_minutes: 10,
    }
}

fn given_stored(repository: &mut MockPreferencesRepository, preferences: Preferences) {
    repository
        .expect_load()
        .returning(move || Ok(preferences.clone()));
}

fn expect_saved_with_theme(repository: &mut MockPreferencesRepository, theme: Theme) {
    repository
        .expect_save()
        .withf(move |preferences| preferences.theme() == theme)
        .times(1)
        .returning(|_| Ok(()));
}

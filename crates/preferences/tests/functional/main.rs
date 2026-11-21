#![allow(clippy::expect_used)]

use cucumber::{given, then, when, World};
use jade_preferences::application::error::PreferencesError;
use jade_preferences::application::get_preferences::GetPreferences;
use jade_preferences::application::remember_vault::{RememberVault, RememberVaultRequest};
use jade_preferences::application::update_preferences::{
    UpdatePreferences, UpdatePreferencesRequest,
};
use jade_preferences::domain::model::Preferences;
use jade_preferences::infrastructure::JsonFilePreferencesRepository;
use jade_shared::application::use_case::UseCase;
use tempfile::TempDir;

#[derive(Debug, World)]
#[world(init = Self::new)]
struct PreferencesWorld {
    folder: TempDir,
    repository: JsonFilePreferencesRepository,
    read: Option<Result<Preferences, PreferencesError>>,
}

impl PreferencesWorld {
    fn new() -> Self {
        let folder = tempfile::tempdir().expect("folder is created");
        let repository = JsonFilePreferencesRepository::new(folder.path().join("preferences.json"));
        Self {
            folder,
            repository,
            read: None,
        }
    }

    fn preferences(&self) -> &Preferences {
        self.read
            .as_ref()
            .expect("preferences were read")
            .as_ref()
            .expect("preferences are readable")
    }
}

#[given("the app has never been started")]
fn never_started(_world: &mut PreferencesWorld) {}

#[given("the preferences file is corrupt")]
fn corrupt_file(world: &mut PreferencesWorld) {
    std::fs::write(world.folder.path().join("preferences.json"), "{not json")
        .expect("file is written");
}

#[when("the app restarts")]
fn the_app_restarts(world: &mut PreferencesWorld) {
    world.repository =
        JsonFilePreferencesRepository::new(world.folder.path().join("preferences.json"));
}

#[when("the preferences are read")]
fn the_preferences_are_read(world: &mut PreferencesWorld) {
    world.read = Some(GetPreferences::new(&world.repository).execute(()));
}

#[when(expr = "the language is set to {string} with theme {string} and sync every {int} minutes")]
fn the_settings_change(world: &mut PreferencesWorld, locale: String, theme: String, minutes: u32) {
    UpdatePreferences::new(&world.repository)
        .execute(UpdatePreferencesRequest {
            locale: Some(locale),
            theme,
            auto_sync_minutes: minutes,
        })
        .expect("preferences are updated");
}

#[when(expr = "the vault {string} is opened")]
fn a_vault_is_opened(world: &mut PreferencesWorld, path: String) {
    RememberVault::new(&world.repository)
        .execute(RememberVaultRequest { path })
        .expect("vault is remembered");
}

#[then("the language follows the system")]
fn the_language_follows_the_system(world: &mut PreferencesWorld) {
    assert_eq!(world.preferences().locale(), None);
}

#[then(expr = "the language is {string}")]
fn the_language_is(world: &mut PreferencesWorld, locale: String) {
    assert_eq!(
        world.preferences().locale().map(|l| l.as_str()),
        Some(locale.as_str())
    );
}

#[then(expr = "the theme is {string}")]
fn the_theme_is(world: &mut PreferencesWorld, theme: String) {
    assert_eq!(world.preferences().theme().as_str(), theme);
}

#[then(expr = "automatic sync runs every {int} minutes")]
fn auto_sync_every(world: &mut PreferencesWorld, minutes: u32) {
    assert_eq!(world.preferences().auto_sync_minutes().value(), minutes);
}

#[then(expr = "the last vault is {string}")]
fn the_last_vault_is(world: &mut PreferencesWorld, path: String) {
    assert_eq!(world.preferences().last_vault(), Some(path.as_str()));
}

#[then(expr = "the recent vaults are {string}")]
fn the_recent_vaults_are(world: &mut PreferencesWorld, paths: String) {
    let expected: Vec<&str> = paths.split(", ").collect();
    assert_eq!(world.preferences().recent_vaults().paths(), expected);
}

#[then("reading the preferences fails")]
fn reading_fails(world: &mut PreferencesWorld) {
    assert!(matches!(
        world.read,
        Some(Err(PreferencesError::Storage(_)))
    ));
}

#[tokio::main]
async fn main() {
    PreferencesWorld::cucumber()
        .fail_on_skipped()
        .run_and_exit(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/features"))
        .await;
}

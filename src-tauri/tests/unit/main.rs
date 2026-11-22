use std::path::Path;

use jade_lib::errors::{ErrorCode, ErrorView};
use jade_lib::watcher::visible_relative;
use jade_notes::application::error::NotesError;
use jade_sync::application::synchronize_vault::SynchronizeVaultError;
use jade_sync::domain::port::VersionControlError;

#[test]
fn reports_changed_notes_relative_to_the_vault() {
    // Arrange
    let root = Path::new("/vaults/notes");

    // Act
    let changed = visible_relative(root, Path::new("/vaults/notes/projects/plan.md"));

    // Assert
    assert_eq!(changed.as_deref(), Some("projects/plan.md"));
}

#[test]
fn ignores_hidden_and_outside_paths() {
    // Arrange
    let root = Path::new("/vaults/notes");
    let paths = [
        "/vaults/notes/.git/index",
        "/vaults/notes/.obsidian/workspace.json",
        "/elsewhere/plan.md",
        "/vaults/notes",
    ];

    // Act
    let changed: Vec<Option<String>> = paths
        .iter()
        .map(|path| visible_relative(root, Path::new(path)))
        .collect();

    // Assert
    assert_eq!(changed, vec![None, None, None, None]);
}

#[test]
fn maps_errors_to_codes_without_user_facing_text() {
    // Arrange
    let errors: Vec<ErrorView> = vec![
        NotesError::VaultNotFound.into(),
        NotesError::InvalidMove.into(),
        SynchronizeVaultError::PushRejected.into(),
        SynchronizeVaultError::VersionControl(VersionControlError::AuthenticationRequired).into(),
        VersionControlError::Network("timeout".to_string()).into(),
    ];

    // Act
    let codes: Vec<ErrorCode> = errors.iter().map(|error| error.code).collect();

    // Assert
    assert_eq!(
        codes,
        vec![
            ErrorCode::VaultNotFound,
            ErrorCode::InvalidMove,
            ErrorCode::PushRejected,
            ErrorCode::AuthRequired,
            ErrorCode::Network,
        ]
    );
}

#[test]
fn carries_the_affected_path_as_a_parameter() {
    // Arrange
    let path = jade_notes::domain::value_object::VaultPath::parse("ideas.md").expect("valid");

    // Act
    let error: ErrorView = NotesError::AlreadyExists(path).into();

    // Assert
    assert_eq!(
        error.params.get("path").map(String::as_str),
        Some("ideas.md")
    );
}

#[test]
fn maps_every_version_control_failure() {
    // Arrange
    let path = jade_sync::domain::value_object::TrackedPath::parse("big.bin").expect("valid");
    let errors = vec![
        VersionControlError::NotARepository,
        VersionControlError::AuthenticationRequired,
        VersionControlError::AuthenticationFailed,
        VersionControlError::Network("down".to_string()),
        VersionControlError::FileTooLarge(path.clone()),
        VersionControlError::PathNotFound(path),
        VersionControlError::Backend("boom".to_string()),
    ];

    // Act
    let codes: Vec<ErrorCode> = errors
        .into_iter()
        .map(|error| ErrorView::from(error).code)
        .collect();

    // Assert
    assert_eq!(
        codes,
        vec![
            ErrorCode::NotARepository,
            ErrorCode::AuthRequired,
            ErrorCode::AuthFailed,
            ErrorCode::Network,
            ErrorCode::FileTooLarge,
            ErrorCode::NotFound,
            ErrorCode::Unexpected,
        ]
    );
}

#[test]
fn maps_every_notes_failure() {
    // Arrange
    let path = jade_notes::domain::value_object::VaultPath::parse("a.md").expect("valid");
    let errors = vec![
        NotesError::InvalidVaultRoot("x".to_string()),
        NotesError::InvalidPath("x".to_string()),
        NotesError::EmptyQuery,
        NotesError::NotFound(path),
        NotesError::Storage("disk".to_string()),
    ];

    // Act
    let codes: Vec<ErrorCode> = errors
        .into_iter()
        .map(|error| ErrorView::from(error).code)
        .collect();

    // Assert
    assert_eq!(
        codes,
        vec![
            ErrorCode::InvalidPath,
            ErrorCode::InvalidPath,
            ErrorCode::EmptyQuery,
            ErrorCode::NotFound,
            ErrorCode::Io,
        ]
    );
}

#[test]
fn maps_preferences_and_credential_failures() {
    // Arrange
    use jade_preferences::application::error::PreferencesError;
    use jade_sync::domain::port::CredentialStoreError;
    let errors: Vec<ErrorView> = vec![
        PreferencesError::InvalidLocale("fr".to_string()).into(),
        PreferencesError::InvalidAutoSync(9999).into(),
        PreferencesError::InvalidVaultPath("x".to_string()).into(),
        PreferencesError::Storage("disk".to_string()).into(),
        CredentialStoreError("locked".to_string()).into(),
    ];

    // Act
    let codes: Vec<ErrorCode> = errors.iter().map(|error| error.code).collect();

    // Assert
    assert_eq!(
        codes,
        vec![
            ErrorCode::InvalidPreference,
            ErrorCode::InvalidPreference,
            ErrorCode::InvalidPath,
            ErrorCode::Io,
            ErrorCode::CredentialsUnavailable,
        ]
    );
}

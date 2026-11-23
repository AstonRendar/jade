use std::collections::BTreeMap;

use jade_notes::application::error::NotesError;
use jade_preferences::application::error::PreferencesError;
use jade_sync::application::clone_repository::CloneRepositoryError;
use jade_sync::application::connect_remote::ConnectRemoteError;
use jade_sync::application::get_repository_status::GetRepositoryStatusError;
use jade_sync::application::init_repository::InitRepositoryError;
use jade_sync::application::list_note_history::ListNoteHistoryError;
use jade_sync::application::read_note_revision::ReadNoteRevisionError;
use jade_sync::application::store_access_token::StoreAccessTokenError;
use jade_sync::application::synchronize_vault::SynchronizeVaultError;
use jade_sync::domain::port::{CredentialStoreError, VersionControlError};
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ErrorCode {
    VaultNotOpen,
    VaultNotFound,
    InvalidPath,
    AlreadyExists,
    NotFound,
    InvalidMove,
    EmptyQuery,
    InvalidUrl,
    InvalidToken,
    InvalidPreference,
    InvalidBase,
    NotARepository,
    AlreadyARepository,
    AuthRequired,
    AuthFailed,
    Network,
    FileTooLarge,
    PushRejected,
    CredentialsUnavailable,
    Io,
    Unexpected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[ts(export)]
pub struct ErrorView {
    pub code: ErrorCode,
    pub params: BTreeMap<String, String>,
}

impl ErrorView {
    pub fn new(code: ErrorCode) -> Self {
        Self {
            code,
            params: BTreeMap::new(),
        }
    }

    pub fn with(code: ErrorCode, key: &str, value: impl Into<String>) -> Self {
        Self {
            code,
            params: BTreeMap::from([(key.to_string(), value.into())]),
        }
    }

    pub fn unexpected(detail: impl Into<String>) -> Self {
        Self::with(ErrorCode::Unexpected, "detail", detail)
    }
}

impl From<NotesError> for ErrorView {
    fn from(error: NotesError) -> Self {
        match error {
            NotesError::InvalidVaultRoot(path) | NotesError::InvalidPath(path) => {
                Self::with(ErrorCode::InvalidPath, "path", path)
            }
            NotesError::VaultNotFound => Self::new(ErrorCode::VaultNotFound),
            NotesError::EmptyQuery => Self::new(ErrorCode::EmptyQuery),
            NotesError::AlreadyExists(path) => {
                Self::with(ErrorCode::AlreadyExists, "path", path.as_str())
            }
            NotesError::NotFound(path) => Self::with(ErrorCode::NotFound, "path", path.as_str()),
            NotesError::InvalidMove => Self::new(ErrorCode::InvalidMove),
            NotesError::InvalidBase(detail) => Self::with(ErrorCode::InvalidBase, "detail", detail),
            NotesError::Storage(detail) => Self::with(ErrorCode::Io, "detail", detail),
        }
    }
}

impl From<VersionControlError> for ErrorView {
    fn from(error: VersionControlError) -> Self {
        match error {
            VersionControlError::NotARepository => Self::new(ErrorCode::NotARepository),
            VersionControlError::AuthenticationRequired => Self::new(ErrorCode::AuthRequired),
            VersionControlError::AuthenticationFailed => Self::new(ErrorCode::AuthFailed),
            VersionControlError::Network(detail) => {
                Self::with(ErrorCode::Network, "detail", detail)
            }
            VersionControlError::FileTooLarge(path) => {
                Self::with(ErrorCode::FileTooLarge, "path", path.as_str())
            }
            VersionControlError::PathNotFound(path) => {
                Self::with(ErrorCode::NotFound, "path", path.as_str())
            }
            VersionControlError::Backend(detail) => Self::unexpected(detail),
        }
    }
}

impl From<CredentialStoreError> for ErrorView {
    fn from(error: CredentialStoreError) -> Self {
        Self::with(ErrorCode::CredentialsUnavailable, "detail", error.0)
    }
}

impl From<PreferencesError> for ErrorView {
    fn from(error: PreferencesError) -> Self {
        match error {
            PreferencesError::InvalidLocale(value) | PreferencesError::InvalidTheme(value) => {
                Self::with(ErrorCode::InvalidPreference, "value", value)
            }
            PreferencesError::InvalidAutoSync(minutes) => {
                Self::with(ErrorCode::InvalidPreference, "value", minutes.to_string())
            }
            PreferencesError::InvalidVaultPath(path) => {
                Self::with(ErrorCode::InvalidPath, "path", path)
            }
            PreferencesError::Storage(detail) => Self::with(ErrorCode::Io, "detail", detail),
        }
    }
}

impl From<SynchronizeVaultError> for ErrorView {
    fn from(error: SynchronizeVaultError) -> Self {
        match error {
            SynchronizeVaultError::InvalidWorkingDirectory(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            SynchronizeVaultError::VersionControl(error) => error.into(),
            SynchronizeVaultError::Credentials(error) => error.into(),
            SynchronizeVaultError::PushRejected => Self::new(ErrorCode::PushRejected),
        }
    }
}

impl From<InitRepositoryError> for ErrorView {
    fn from(error: InitRepositoryError) -> Self {
        match error {
            InitRepositoryError::InvalidWorkingDirectory(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            InitRepositoryError::AlreadyARepository => Self::new(ErrorCode::AlreadyARepository),
            InitRepositoryError::VersionControl(error) => error.into(),
        }
    }
}

impl From<CloneRepositoryError> for ErrorView {
    fn from(error: CloneRepositoryError) -> Self {
        match error {
            CloneRepositoryError::InvalidWorkingDirectory(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            CloneRepositoryError::InvalidRemoteUrl(error) => {
                Self::with(ErrorCode::InvalidUrl, "url", error.0)
            }
            CloneRepositoryError::VersionControl(error) => error.into(),
            CloneRepositoryError::Credentials(error) => error.into(),
        }
    }
}

impl From<ConnectRemoteError> for ErrorView {
    fn from(error: ConnectRemoteError) -> Self {
        match error {
            ConnectRemoteError::InvalidWorkingDirectory(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            ConnectRemoteError::InvalidRemoteUrl(error) => {
                Self::with(ErrorCode::InvalidUrl, "url", error.0)
            }
            ConnectRemoteError::NotARepository => Self::new(ErrorCode::NotARepository),
            ConnectRemoteError::VersionControl(error) => error.into(),
        }
    }
}

impl From<GetRepositoryStatusError> for ErrorView {
    fn from(error: GetRepositoryStatusError) -> Self {
        match error {
            GetRepositoryStatusError::InvalidWorkingDirectory(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            GetRepositoryStatusError::VersionControl(error) => error.into(),
        }
    }
}

impl From<ListNoteHistoryError> for ErrorView {
    fn from(error: ListNoteHistoryError) -> Self {
        match error {
            ListNoteHistoryError::InvalidWorkingDirectory(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            ListNoteHistoryError::InvalidPath(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            ListNoteHistoryError::VersionControl(error) => error.into(),
        }
    }
}

impl From<ReadNoteRevisionError> for ErrorView {
    fn from(error: ReadNoteRevisionError) -> Self {
        match error {
            ReadNoteRevisionError::InvalidWorkingDirectory(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            ReadNoteRevisionError::InvalidPath(error) => {
                Self::with(ErrorCode::InvalidPath, "path", error.0)
            }
            ReadNoteRevisionError::InvalidCommitId(error) => {
                Self::with(ErrorCode::NotFound, "commit", error.0)
            }
            ReadNoteRevisionError::VersionControl(error) => error.into(),
        }
    }
}

impl From<StoreAccessTokenError> for ErrorView {
    fn from(error: StoreAccessTokenError) -> Self {
        match error {
            StoreAccessTokenError::InvalidToken(_) => Self::new(ErrorCode::InvalidToken),
            StoreAccessTokenError::Credentials(error) => error.into(),
        }
    }
}

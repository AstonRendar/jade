use crate::domain::port::NoteRepositoryError;
use crate::domain::value_object::{
    EmptySearchQueryError, InvalidNotePathError, InvalidVaultPathError, InvalidVaultRootError,
    VaultPath, VaultRoot,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotesError {
    #[error("invalid vault root `{0}`")]
    InvalidVaultRoot(String),
    #[error("the vault folder does not exist")]
    VaultNotFound,
    #[error("invalid path `{0}`")]
    InvalidPath(String),
    #[error("the search query is empty")]
    EmptyQuery,
    #[error("`{0:?}` already exists")]
    AlreadyExists(VaultPath),
    #[error("`{0:?}` not found")]
    NotFound(VaultPath),
    #[error("a folder cannot be moved inside itself")]
    InvalidMove,
    #[error("storage failure: {0}")]
    Storage(String),
}

impl From<InvalidVaultRootError> for NotesError {
    fn from(error: InvalidVaultRootError) -> Self {
        Self::InvalidVaultRoot(error.0)
    }
}

impl From<InvalidVaultPathError> for NotesError {
    fn from(error: InvalidVaultPathError) -> Self {
        Self::InvalidPath(error.0)
    }
}

impl From<InvalidNotePathError> for NotesError {
    fn from(error: InvalidNotePathError) -> Self {
        Self::InvalidPath(error.0)
    }
}

impl From<EmptySearchQueryError> for NotesError {
    fn from(_: EmptySearchQueryError) -> Self {
        Self::EmptyQuery
    }
}

impl From<NoteRepositoryError> for NotesError {
    fn from(error: NoteRepositoryError) -> Self {
        match error {
            NoteRepositoryError::NotFound(path) => Self::NotFound(path),
            NoteRepositoryError::Io(message) => Self::Storage(message),
        }
    }
}

pub(crate) fn vault_root(raw: &str) -> Result<VaultRoot, NotesError> {
    Ok(VaultRoot::parse(raw)?)
}

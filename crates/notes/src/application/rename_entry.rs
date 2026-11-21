use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::value_object::VaultPath;

pub struct RenameEntryRequest {
    pub vault_root: String,
    pub from: String,
    pub to: String,
}

pub struct RenameEntry<R> {
    repository: R,
}

impl<R: NoteRepository> RenameEntry<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for RenameEntry<R> {
    type Request = RenameEntryRequest;
    type Response = VaultPath;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let from = VaultPath::parse(&request.from)?;
        let to = VaultPath::parse(&request.to)?;
        if to.is_within(&from) {
            return Err(NotesError::InvalidMove);
        }
        if self.repository.entry_exists(&root, &to)? {
            return Err(NotesError::AlreadyExists(to));
        }
        self.repository.rename(&root, &from, &to)?;
        Ok(to)
    }
}

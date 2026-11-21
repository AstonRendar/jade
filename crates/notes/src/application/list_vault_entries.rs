use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::model::VaultEntry;
use crate::domain::port::NoteRepository;

pub struct ListVaultEntriesRequest {
    pub vault_root: String,
}

pub struct ListVaultEntries<R> {
    repository: R,
}

impl<R: NoteRepository> ListVaultEntries<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for ListVaultEntries<R> {
    type Request = ListVaultEntriesRequest;
    type Response = Vec<VaultEntry>;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let mut entries = self.repository.list(&vault_root(&request.vault_root)?)?;
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }
}

use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::value_object::VaultPath;

pub struct DeleteEntryRequest {
    pub vault_root: String,
    pub path: String,
}

pub struct DeleteEntry<R> {
    repository: R,
}

impl<R: NoteRepository> DeleteEntry<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for DeleteEntry<R> {
    type Request = DeleteEntryRequest;
    type Response = ();
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let path = VaultPath::parse(&request.path)?;
        Ok(self.repository.delete(&root, &path)?)
    }
}

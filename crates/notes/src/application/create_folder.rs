use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::value_object::VaultPath;

pub struct CreateFolderRequest {
    pub vault_root: String,
    pub path: String,
}

pub struct CreateFolder<R> {
    repository: R,
}

impl<R: NoteRepository> CreateFolder<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for CreateFolder<R> {
    type Request = CreateFolderRequest;
    type Response = VaultPath;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let path = VaultPath::parse(&request.path)?;
        if self.repository.entry_exists(&root, &path)? {
            return Err(NotesError::AlreadyExists(path));
        }
        self.repository.create_folder(&root, &path)?;
        Ok(path)
    }
}

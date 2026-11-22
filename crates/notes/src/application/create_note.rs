use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::value_object::{NotePath, VaultPath};

pub struct CreateNoteRequest {
    pub vault_root: String,
    pub folder: Option<String>,
    pub title: String,
}

pub struct CreateNote<R> {
    repository: R,
}

impl<R: NoteRepository> CreateNote<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for CreateNote<R> {
    type Request = CreateNoteRequest;
    type Response = NotePath;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let folder = request
            .folder
            .as_deref()
            .map(VaultPath::parse)
            .transpose()?;
        let path = NotePath::from_title(folder.as_ref(), &request.title)?;
        if self.repository.entry_exists(&root, path.as_vault_path())? {
            return Err(NotesError::AlreadyExists(path.as_vault_path().clone()));
        }
        self.repository.write(&root, &path, "")?;
        Ok(path)
    }
}

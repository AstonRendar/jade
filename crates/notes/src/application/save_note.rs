use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::value_object::NotePath;

pub struct SaveNoteRequest {
    pub vault_root: String,
    pub path: String,
    pub content: String,
}

pub struct SaveNote<R> {
    repository: R,
}

impl<R: NoteRepository> SaveNote<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for SaveNote<R> {
    type Request = SaveNoteRequest;
    type Response = ();
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let path = NotePath::parse(&request.path)?;
        Ok(self.repository.write(&root, &path, &request.content)?)
    }
}

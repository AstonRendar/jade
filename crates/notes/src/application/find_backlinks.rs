use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::service::{Backlink, BacklinkFinder};
use crate::domain::value_object::NotePath;

pub struct FindBacklinksRequest {
    pub vault_root: String,
    pub path: String,
}

pub struct FindBacklinks<R> {
    repository: R,
}

impl<R: NoteRepository> FindBacklinks<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for FindBacklinks<R> {
    type Request = FindBacklinksRequest;
    type Response = Vec<Backlink>;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let path = NotePath::parse(&request.path)?;
        let notes = self.repository.notes(&root)?;
        Ok(BacklinkFinder.find(&path, &notes))
    }
}

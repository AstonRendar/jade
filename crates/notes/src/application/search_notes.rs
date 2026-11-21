use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::service::{NoteSearch, SearchHit};
use crate::domain::value_object::SearchQuery;

pub struct SearchNotesRequest {
    pub vault_root: String,
    pub query: String,
}

pub struct SearchNotes<R> {
    repository: R,
}

impl<R: NoteRepository> SearchNotes<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for SearchNotes<R> {
    type Request = SearchNotesRequest;
    type Response = Vec<SearchHit>;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let query = SearchQuery::parse(&request.query)?;
        let notes = self.repository.notes(&root)?;
        Ok(NoteSearch::default().search(&query, &notes))
    }
}

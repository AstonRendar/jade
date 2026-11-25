use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::model::EntryKind;
use crate::domain::port::NoteRepository;
use crate::domain::service::LinkResolver;
use crate::domain::value_object::NotePath;

pub struct ResolveLinksRequest {
    pub vault_root: String,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLink {
    pub target: String,
    pub note: Option<NotePath>,
}

pub struct ResolveLinks<R> {
    repository: R,
}

impl<R: NoteRepository> ResolveLinks<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for ResolveLinks<R> {
    type Request = ResolveLinksRequest;
    type Response = Vec<ResolvedLink>;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let paths: Vec<NotePath> = self
            .repository
            .list(&root)?
            .into_iter()
            .filter(|entry| entry.kind == EntryKind::Note)
            .filter_map(|entry| NotePath::from_vault_path(entry.path).ok())
            .collect();
        let resolver = LinkResolver::new(&paths);
        Ok(request
            .targets
            .into_iter()
            .map(|target| ResolvedLink {
                note: resolver.resolve(&target),
                target,
            })
            .collect())
    }
}

use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::model::EntryKind;
use crate::domain::port::NoteRepository;
use crate::domain::service::FileResolver;
use crate::domain::value_object::VaultPath;

pub struct ResolveFilesRequest {
    pub vault_root: String,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFile {
    pub target: String,
    pub path: Option<VaultPath>,
}

pub struct ResolveFiles<R> {
    repository: R,
}

impl<R: NoteRepository> ResolveFiles<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for ResolveFiles<R> {
    type Request = ResolveFilesRequest;
    type Response = Vec<ResolvedFile>;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let files: Vec<VaultPath> = self
            .repository
            .list(&root)?
            .into_iter()
            .filter(|entry| entry.kind != EntryKind::Folder)
            .map(|entry| entry.path)
            .collect();
        let resolver = FileResolver::new(&files);
        Ok(request
            .targets
            .into_iter()
            .map(|target| ResolvedFile {
                path: resolver.resolve(&target),
                target,
            })
            .collect())
    }
}

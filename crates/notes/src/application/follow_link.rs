use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::service::LinkResolver;
use crate::domain::value_object::NotePath;

const EXTENSION: &str = ".md";

pub struct FollowLinkRequest {
    pub vault_root: String,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FollowedLink {
    pub note: NotePath,
    pub created: bool,
}

pub struct FollowLink<R> {
    repository: R,
}

impl<R: NoteRepository> FollowLink<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for FollowLink<R> {
    type Request = FollowLinkRequest;
    type Response = FollowedLink;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let paths: Vec<NotePath> = self
            .repository
            .notes(&root)?
            .into_iter()
            .map(|note| note.path().clone())
            .collect();
        if let Some(note) = LinkResolver.resolve(&request.target, &paths) {
            return Ok(FollowedLink {
                note,
                created: false,
            });
        }
        let note = note_path_for(&request.target)?;
        self.repository.write(&root, &note, "")?;
        Ok(FollowedLink {
            note,
            created: true,
        })
    }
}

fn note_path_for(target: &str) -> Result<NotePath, NotesError> {
    let target = target.trim();
    let has_extension = target.to_lowercase().ends_with(EXTENSION);
    let raw = if has_extension {
        target.to_string()
    } else {
        format!("{target}{EXTENSION}")
    };
    Ok(NotePath::parse(&raw)?)
}

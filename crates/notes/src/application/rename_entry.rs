use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::model::Note;
use crate::domain::port::NoteRepository;
use crate::domain::service::{LinkRewriter, NoteMove};
use crate::domain::value_object::{NotePath, VaultPath, VaultRoot};

pub struct RenameEntryRequest {
    pub vault_root: String,
    pub from: String,
    pub to: String,
}

pub struct RenameEntry<R> {
    repository: R,
}

impl<R: NoteRepository> RenameEntry<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    fn update_links(
        &self,
        root: &VaultRoot,
        notes: &[Note],
        moves: &[NoteMove],
    ) -> Result<(), NotesError> {
        let before: Vec<NotePath> = notes.iter().map(|note| note.path().clone()).collect();
        for note in notes {
            if let Some(content) = LinkRewriter.rewrite(note.content(), moves, &before) {
                let location = moves
                    .iter()
                    .find(|note_move| &note_move.from == note.path())
                    .map_or(note.path(), |note_move| &note_move.to);
                self.repository.write(root, location, &content)?;
            }
        }
        Ok(())
    }
}

impl<R: NoteRepository> UseCase for RenameEntry<R> {
    type Request = RenameEntryRequest;
    type Response = VaultPath;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let from = VaultPath::parse(&request.from)?;
        let to = VaultPath::parse(&request.to)?;
        if to.is_within(&from) {
            return Err(NotesError::InvalidMove);
        }
        if self.repository.entry_exists(&root, &to)? {
            return Err(NotesError::AlreadyExists(to));
        }
        let notes = self.repository.notes(&root)?;
        let moves = moves_of(&from, &to, &notes);
        self.repository.rename(&root, &from, &to)?;
        self.update_links(&root, &notes, &moves)?;
        Ok(to)
    }
}

fn moves_of(from: &VaultPath, to: &VaultPath, notes: &[Note]) -> Vec<NoteMove> {
    notes
        .iter()
        .filter(|note| note.path().as_vault_path().is_within(from))
        .filter_map(|note| {
            let rest = &note.path().as_str()[from.as_str().len()..];
            let moved = VaultPath::parse(&format!("{}{rest}", to.as_str())).ok()?;
            Some(NoteMove {
                from: note.path().clone(),
                to: NotePath::from_vault_path(moved).ok()?,
            })
        })
        .collect()
}

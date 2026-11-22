use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;

/// Deletes a vault folder, only when it sits directly inside `container`.
pub struct DeleteVaultRequest {
    pub vault_root: String,
    pub container: String,
}

pub struct DeleteVault<R> {
    repository: R,
}

impl<R: NoteRepository> DeleteVault<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for DeleteVault<R> {
    type Request = DeleteVaultRequest;
    type Response = ();
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let container = vault_root(&request.container)?;
        if root.as_path().parent() != Some(container.as_path()) {
            return Err(NotesError::InvalidPath(request.vault_root));
        }
        if !self.repository.vault_exists(&root) {
            return Err(NotesError::VaultNotFound);
        }
        Ok(self.repository.delete_root(&root)?)
    }
}

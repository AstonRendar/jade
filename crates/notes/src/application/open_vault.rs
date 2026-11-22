use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::NoteRepository;
use crate::domain::value_object::VaultRoot;

pub struct OpenVaultRequest {
    pub vault_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultInfo {
    pub root: VaultRoot,
    pub name: String,
}

pub struct OpenVault<R> {
    repository: R,
}

impl<R: NoteRepository> OpenVault<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for OpenVault<R> {
    type Request = OpenVaultRequest;
    type Response = VaultInfo;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        if !self.repository.vault_exists(&root) {
            return Err(NotesError::VaultNotFound);
        }
        Ok(VaultInfo {
            name: root.name(),
            root,
        })
    }
}

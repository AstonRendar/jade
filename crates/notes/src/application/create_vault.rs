use std::path::Path;

use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use super::open_vault::VaultInfo;
use crate::domain::port::NoteRepository;
use crate::domain::value_object::VaultPath;

pub struct CreateVaultRequest {
    pub parent: String,
    pub name: String,
}

pub struct CreateVault<R> {
    repository: R,
}

impl<R: NoteRepository> CreateVault<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

impl<R: NoteRepository> UseCase for CreateVault<R> {
    type Request = CreateVaultRequest;
    type Response = VaultInfo;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let name = folder_name(&request.name)?;
        let location = Path::new(&request.parent).join(name.as_str());
        let root = vault_root(&location.display().to_string())?;
        if self.repository.vault_exists(&root) {
            return Err(NotesError::AlreadyExists(name));
        }
        self.repository.create_root(&root)?;
        Ok(VaultInfo {
            name: root.name(),
            root,
        })
    }
}

fn folder_name(raw: &str) -> Result<VaultPath, NotesError> {
    let name = VaultPath::parse(raw.trim())?;
    if name.parent().is_some() || name.is_hidden() {
        return Err(NotesError::InvalidPath(raw.to_string()));
    }
    Ok(name)
}

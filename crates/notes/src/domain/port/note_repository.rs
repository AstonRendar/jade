use crate::domain::model::{Note, VaultEntry};
use crate::domain::value_object::{NotePath, VaultPath, VaultRoot};

pub trait NoteRepository {
    fn vault_exists(&self, root: &VaultRoot) -> bool;
    fn create_root(&self, root: &VaultRoot) -> Result<(), NoteRepositoryError>;
    fn entry_exists(&self, root: &VaultRoot, path: &VaultPath)
        -> Result<bool, NoteRepositoryError>;
    fn list(&self, root: &VaultRoot) -> Result<Vec<VaultEntry>, NoteRepositoryError>;
    fn read(&self, root: &VaultRoot, path: &NotePath) -> Result<String, NoteRepositoryError>;
    fn write(
        &self,
        root: &VaultRoot,
        path: &NotePath,
        content: &str,
    ) -> Result<(), NoteRepositoryError>;
    fn create_folder(&self, root: &VaultRoot, path: &VaultPath) -> Result<(), NoteRepositoryError>;
    fn rename(
        &self,
        root: &VaultRoot,
        from: &VaultPath,
        to: &VaultPath,
    ) -> Result<(), NoteRepositoryError>;
    fn delete(&self, root: &VaultRoot, path: &VaultPath) -> Result<(), NoteRepositoryError>;
    fn notes(&self, root: &VaultRoot) -> Result<Vec<Note>, NoteRepositoryError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoteRepositoryError {
    #[error("`{0:?}` not found")]
    NotFound(VaultPath),
    #[error("storage failure: {0}")]
    Io(String),
}

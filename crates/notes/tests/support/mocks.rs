use jade_notes::domain::model::{Note, VaultEntry};
use jade_notes::domain::port::{NoteRepository, NoteRepositoryError};
use jade_notes::domain::value_object::{NotePath, VaultPath, VaultRoot};
use mockall::mock;

mock! {
    pub NoteRepository {}

    impl NoteRepository for NoteRepository {
        fn vault_exists(&self, root: &VaultRoot) -> bool;
        fn entry_exists(&self, root: &VaultRoot, path: &VaultPath) -> Result<bool, NoteRepositoryError>;
        fn list(&self, root: &VaultRoot) -> Result<Vec<VaultEntry>, NoteRepositoryError>;
        fn read(&self, root: &VaultRoot, path: &NotePath) -> Result<String, NoteRepositoryError>;
        fn write(&self, root: &VaultRoot, path: &NotePath, content: &str) -> Result<(), NoteRepositoryError>;
        fn create_folder(&self, root: &VaultRoot, path: &VaultPath) -> Result<(), NoteRepositoryError>;
        fn rename(&self, root: &VaultRoot, from: &VaultPath, to: &VaultPath) -> Result<(), NoteRepositoryError>;
        fn delete(&self, root: &VaultRoot, path: &VaultPath) -> Result<(), NoteRepositoryError>;
        fn notes(&self, root: &VaultRoot) -> Result<Vec<Note>, NoteRepositoryError>;
    }
}

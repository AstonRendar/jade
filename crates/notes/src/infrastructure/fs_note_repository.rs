use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::domain::model::{EntryKind, FileMetadata, Note, VaultEntry};
use crate::domain::port::{NoteRepository, NoteRepositoryError};
use crate::domain::value_object::{NotePath, VaultPath, VaultRoot};

const TEMPORARY_SUFFIX: &str = ".jade-tmp";

#[derive(Debug, Clone, Copy, Default)]
pub struct FsNoteRepository;

impl NoteRepository for FsNoteRepository {
    fn vault_exists(&self, root: &VaultRoot) -> bool {
        root.as_path().is_dir()
    }

    fn create_root(&self, root: &VaultRoot) -> Result<(), NoteRepositoryError> {
        fs::create_dir_all(root.as_path()).map_err(storage)
    }

    fn delete_root(&self, root: &VaultRoot) -> Result<(), NoteRepositoryError> {
        fs::remove_dir_all(root.as_path()).map_err(storage)
    }

    fn entry_exists(
        &self,
        root: &VaultRoot,
        path: &VaultPath,
    ) -> Result<bool, NoteRepositoryError> {
        Ok(location(root, path).exists())
    }

    fn list(&self, root: &VaultRoot) -> Result<Vec<VaultEntry>, NoteRepositoryError> {
        let mut entries = Vec::new();
        collect(root.as_path(), root.as_path(), &mut entries).map_err(storage)?;
        Ok(entries)
    }

    fn read(&self, root: &VaultRoot, path: &NotePath) -> Result<String, NoteRepositoryError> {
        fs::read_to_string(location(root, path.as_vault_path()))
            .map_err(|error| failure(error, path.as_vault_path()))
    }

    fn write(
        &self,
        root: &VaultRoot,
        path: &NotePath,
        content: &str,
    ) -> Result<(), NoteRepositoryError> {
        self.write_text(root, path.as_vault_path(), content)
    }

    fn create_folder(&self, root: &VaultRoot, path: &VaultPath) -> Result<(), NoteRepositoryError> {
        fs::create_dir_all(location(root, path)).map_err(storage)
    }

    fn rename(
        &self,
        root: &VaultRoot,
        from: &VaultPath,
        to: &VaultPath,
    ) -> Result<(), NoteRepositoryError> {
        let source = location(root, from);
        if !source.exists() {
            return Err(NoteRepositoryError::NotFound(from.clone()));
        }
        let target = location(root, to);
        if let Some(folder) = target.parent() {
            fs::create_dir_all(folder).map_err(storage)?;
        }
        fs::rename(source, target).map_err(|error| failure(error, from))
    }

    fn delete(&self, root: &VaultRoot, path: &VaultPath) -> Result<(), NoteRepositoryError> {
        let target = location(root, path);
        if target.is_dir() {
            fs::remove_dir_all(target)
        } else {
            fs::remove_file(target)
        }
        .map_err(|error| failure(error, path))
    }

    fn notes(&self, root: &VaultRoot) -> Result<Vec<Note>, NoteRepositoryError> {
        self.list(root)?
            .into_iter()
            .filter(|entry| entry.kind == EntryKind::Note)
            .filter_map(|entry| NotePath::from_vault_path(entry.path).ok())
            .map(|path| {
                let content = self.read(root, &path)?;
                Ok(Note::new(path, content))
            })
            .collect()
    }

    fn read_text(&self, root: &VaultRoot, path: &VaultPath) -> Result<String, NoteRepositoryError> {
        fs::read_to_string(location(root, path)).map_err(|error| failure(error, path))
    }

    fn write_text(
        &self,
        root: &VaultRoot,
        path: &VaultPath,
        content: &str,
    ) -> Result<(), NoteRepositoryError> {
        let file = location(root, path);
        let folder = file.parent().unwrap_or(root.as_path());
        fs::create_dir_all(folder).map_err(storage)?;
        let temporary = folder.join(format!(".{}{TEMPORARY_SUFFIX}", path.file_name()));
        fs::write(&temporary, content).map_err(storage)?;
        fs::rename(&temporary, &file).map_err(storage)
    }

    fn metadata(
        &self,
        root: &VaultRoot,
        path: &VaultPath,
    ) -> Result<FileMetadata, NoteRepositoryError> {
        let metadata = fs::metadata(location(root, path)).map_err(|error| failure(error, path))?;
        let modified_ms = metadata.modified().map_or(0, milliseconds);
        Ok(FileMetadata {
            size: metadata.len(),
            created_ms: metadata.created().map_or(modified_ms, milliseconds),
            modified_ms,
        })
    }
}

fn milliseconds(time: SystemTime) -> i64 {
    time.duration_since(UNIX_EPOCH).map_or(0, |elapsed| {
        i64::try_from(elapsed.as_millis()).unwrap_or(i64::MAX)
    })
}

fn location(root: &VaultRoot, path: &VaultPath) -> PathBuf {
    root.as_path().join(path.as_str())
}

fn collect(root: &Path, folder: &Path, entries: &mut Vec<VaultEntry>) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let full = entry?.path();
        if full
            .file_name()
            .is_some_and(|name| name.to_string_lossy().starts_with('.'))
        {
            continue;
        }
        let Some(path) = vault_path_of(root, &full) else {
            continue;
        };
        if full.is_dir() {
            entries.push(VaultEntry {
                path,
                kind: EntryKind::Folder,
            });
            collect(root, &full, entries)?;
        } else {
            let kind = if NotePath::from_vault_path(path.clone()).is_ok() {
                EntryKind::Note
            } else {
                EntryKind::File
            };
            entries.push(VaultEntry { path, kind });
        }
    }
    Ok(())
}

fn vault_path_of(root: &Path, full: &Path) -> Option<VaultPath> {
    let relative = full.strip_prefix(root).ok()?;
    let segments: Vec<String> = relative
        .iter()
        .map(|segment| segment.to_string_lossy().into_owned())
        .collect();
    VaultPath::parse(&segments.join("/")).ok()
}

fn failure(error: io::Error, path: &VaultPath) -> NoteRepositoryError {
    if error.kind() == io::ErrorKind::NotFound {
        NoteRepositoryError::NotFound(path.clone())
    } else {
        storage(error)
    }
}

fn storage(error: io::Error) -> NoteRepositoryError {
    NoteRepositoryError::Io(error.to_string())
}

use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::model::EntryKind;
use crate::domain::port::{DataFormat, NoteRepository};
use crate::domain::service::FileResolver;
use crate::domain::value_object::{AttachmentFolder, Data, VaultPath, VaultRoot};

const SETTINGS_FILE: &str = ".obsidian/app.json";
const FOLDER_SETTING: &str = "attachmentFolderPath";

pub struct SaveAttachmentRequest {
    pub vault_root: String,
    pub note: String,
    pub file_name: String,
    pub content: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedAttachment {
    pub path: VaultPath,
    pub link: String,
}

pub struct SaveAttachment<R, F> {
    repository: R,
    format: F,
}

impl<R: NoteRepository, F: DataFormat> SaveAttachment<R, F> {
    pub fn new(repository: R, format: F) -> Self {
        Self { repository, format }
    }

    fn attachment_folder(&self, root: &VaultRoot) -> AttachmentFolder {
        let setting = VaultPath::parse(SETTINGS_FILE)
            .ok()
            .and_then(|path| self.repository.read_text(root, &path).ok())
            .and_then(|text| self.format.parse(&text).ok());
        AttachmentFolder::from_setting(
            setting
                .as_ref()
                .and_then(|data| data.get(FOLDER_SETTING))
                .and_then(Data::as_text),
        )
    }

    fn available_path(
        &self,
        root: &VaultRoot,
        folder: Option<&VaultPath>,
        file_name: &str,
    ) -> Result<VaultPath, NotesError> {
        let (stem, extension) = match file_name.rsplit_once('.') {
            Some((stem, extension)) if !stem.is_empty() => (stem, format!(".{extension}")),
            _ => (file_name, String::new()),
        };
        for attempt in 0.. {
            let name = if attempt == 0 {
                file_name.to_string()
            } else {
                format!("{stem} {attempt}{extension}")
            };
            let path = match folder {
                Some(folder) => folder.join(&name)?,
                None => VaultPath::parse(&name)?,
            };
            if !self.repository.entry_exists(root, &path)? {
                return Ok(path);
            }
        }
        Err(NotesError::AlreadyExists(VaultPath::parse(file_name)?))
    }

    fn shortest_link(&self, root: &VaultRoot, path: &VaultPath) -> Result<String, NotesError> {
        let files: Vec<VaultPath> = self
            .repository
            .list(root)?
            .into_iter()
            .filter(|entry| entry.kind != EntryKind::Folder)
            .map(|entry| entry.path)
            .collect();
        let name = path.file_name();
        if FileResolver::new(&files).resolve(name).as_ref() == Some(path) {
            Ok(name.to_string())
        } else {
            Ok(path.as_str().to_string())
        }
    }
}

impl<R: NoteRepository, F: DataFormat> UseCase for SaveAttachment<R, F> {
    type Request = SaveAttachmentRequest;
    type Response = SavedAttachment;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let note = VaultPath::parse(&request.note)?;
        let file_name = request.file_name.trim();
        if file_name.is_empty() || file_name.contains('/') {
            return Err(NotesError::InvalidPath(request.file_name));
        }
        let folder = self.attachment_folder(&root).folder_for(&note);
        let path = self.available_path(&root, folder.as_ref(), file_name)?;
        self.repository
            .write_bytes(&root, &path, &request.content)?;
        let link = self.shortest_link(&root, &path)?;
        Ok(SavedAttachment { path, link })
    }
}

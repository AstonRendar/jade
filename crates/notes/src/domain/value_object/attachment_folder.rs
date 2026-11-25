use super::VaultPath;

const ROOT: &str = "/";
const NOTE_FOLDER: &str = "./";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachmentFolder {
    Root,
    NoteFolder,
    NoteSubfolder(String),
    Folder(VaultPath),
}

impl AttachmentFolder {
    pub fn from_setting(setting: Option<&str>) -> Self {
        let setting = setting.unwrap_or(ROOT).trim();
        if setting.is_empty() || setting == ROOT {
            return Self::Root;
        }
        if setting == NOTE_FOLDER || setting == "." {
            return Self::NoteFolder;
        }
        if let Some(subfolder) = setting.strip_prefix(NOTE_FOLDER) {
            return Self::NoteSubfolder(subfolder.trim_matches('/').to_string());
        }
        VaultPath::parse(setting.trim_matches('/')).map_or(Self::Root, Self::Folder)
    }

    pub fn folder_for(&self, note: &VaultPath) -> Option<VaultPath> {
        match self {
            Self::Root => None,
            Self::NoteFolder => note.parent(),
            Self::NoteSubfolder(subfolder) => {
                let joined = match note.parent() {
                    Some(parent) => format!("{}/{subfolder}", parent.as_str()),
                    None => subfolder.clone(),
                };
                VaultPath::parse(&joined).ok().or_else(|| note.parent())
            }
            Self::Folder(folder) => Some(folder.clone()),
        }
    }
}

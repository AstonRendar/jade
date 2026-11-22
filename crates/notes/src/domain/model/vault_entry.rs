use crate::domain::value_object::VaultPath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryKind {
    Folder,
    Note,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VaultEntry {
    pub path: VaultPath,
    pub kind: EntryKind,
}

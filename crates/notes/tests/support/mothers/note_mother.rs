use jade_notes::domain::model::Note;
use jade_notes::domain::value_object::{NotePath, VaultPath, VaultRoot};

pub const VAULT: &str = "/vaults/notes";

pub struct NoteMother;

impl NoteMother {
    pub fn root() -> VaultRoot {
        VaultRoot::parse(VAULT).expect("valid vault root")
    }

    pub fn path(raw: &str) -> NotePath {
        NotePath::parse(raw).expect("valid note path")
    }

    pub fn vault_path(raw: &str) -> VaultPath {
        VaultPath::parse(raw).expect("valid vault path")
    }

    pub fn paths(raws: &[&str]) -> Vec<NotePath> {
        raws.iter().map(|raw| Self::path(raw)).collect()
    }

    pub fn note(path: &str, content: &str) -> Note {
        Note::new(Self::path(path), content.to_string())
    }
}

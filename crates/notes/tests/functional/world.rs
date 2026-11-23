use jade_notes::application::error::NotesError;
use jade_notes::application::follow_link::FollowedLink;
use jade_notes::application::open_vault::VaultInfo;
use jade_notes::application::query_base::QueryBaseResponse;
use jade_notes::application::resolve_links::ResolvedLink;
use jade_notes::domain::service::{Backlink, SearchHit};
use tempfile::TempDir;

#[derive(Debug, cucumber::World)]
#[world(init = Self::new)]
pub struct NotesWorld {
    vault: TempDir,
    pub opened: Option<VaultInfo>,
    pub last_error: Option<NotesError>,
    pub followed: Option<FollowedLink>,
    pub backlinks: Vec<Backlink>,
    pub resolved: Vec<ResolvedLink>,
    pub hits: Vec<SearchHit>,
    pub last_vault_name: Option<String>,
    pub base: Option<QueryBaseResponse>,
}

impl NotesWorld {
    fn new() -> Self {
        Self {
            vault: tempfile::tempdir().expect("vault folder is created"),
            opened: None,
            last_error: None,
            followed: None,
            backlinks: Vec::new(),
            resolved: Vec::new(),
            hits: Vec::new(),
            last_vault_name: None,
            base: None,
        }
    }

    pub fn root(&self) -> String {
        self.vault.path().display().to_string()
    }

    pub fn folder_name(&self) -> String {
        self.vault
            .path()
            .file_name()
            .expect("vault has a name")
            .to_string_lossy()
            .into_owned()
    }

    pub fn record<T>(&mut self, result: Result<T, NotesError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(error) => {
                self.last_error = Some(error);
                None
            }
        }
    }
}

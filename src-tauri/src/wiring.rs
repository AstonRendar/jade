use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use jade_notes::application::create_folder::{CreateFolder, CreateFolderRequest};
use jade_notes::application::create_note::{CreateNote, CreateNoteRequest};
use jade_notes::application::create_vault::{CreateVault, CreateVaultRequest};
use jade_notes::application::delete_entry::{DeleteEntry, DeleteEntryRequest};
use jade_notes::application::delete_vault::{DeleteVault, DeleteVaultRequest};
use jade_notes::application::error::NotesError;
use jade_notes::application::find_backlinks::{FindBacklinks, FindBacklinksRequest};
use jade_notes::application::follow_link::{FollowLink, FollowLinkRequest};
use jade_notes::application::list_properties::{ListProperties, ListPropertiesRequest};
use jade_notes::application::list_vault_entries::{ListVaultEntries, ListVaultEntriesRequest};
use jade_notes::application::open_vault::{OpenVault, OpenVaultRequest};
use jade_notes::application::query_base::{QueryBase, QueryBaseRequest};
use jade_notes::application::read_note::{ReadNote, ReadNoteRequest};
use jade_notes::application::read_properties::{ReadProperties, ReadPropertiesRequest};
use jade_notes::application::rename_entry::{RenameEntry, RenameEntryRequest};
use jade_notes::application::resolve_links::{ResolveLinks, ResolveLinksRequest};
use jade_notes::application::save_note::{SaveNote, SaveNoteRequest};
use jade_notes::application::search_notes::{SearchNotes, SearchNotesRequest};
use jade_notes::application::write_properties::{WriteProperties, WritePropertiesRequest};
use jade_notes::infrastructure::{FsNoteRepository, YamlDataFormat};
use jade_preferences::application::forget_vault::{ForgetVault, ForgetVaultRequest};
use jade_preferences::application::get_preferences::GetPreferences;
use jade_preferences::application::remember_vault::{RememberVault, RememberVaultRequest};
use jade_preferences::application::update_preferences::{
    UpdatePreferences, UpdatePreferencesRequest,
};
use jade_preferences::infrastructure::JsonFilePreferencesRepository;
use jade_shared::application::use_case::UseCase;
use jade_sync::application::check_access_token::CheckAccessToken;
use jade_sync::application::clone_repository::{CloneRepository, CloneRepositoryRequest};
use jade_sync::application::connect_remote::{ConnectRemote, ConnectRemoteRequest};
use jade_sync::application::get_repository_status::{
    GetRepositoryStatus, GetRepositoryStatusRequest,
};
use jade_sync::application::init_repository::{InitRepository, InitRepositoryRequest};
use jade_sync::application::list_note_history::{ListNoteHistory, ListNoteHistoryRequest};
use jade_sync::application::read_note_revision::{ReadNoteRevision, ReadNoteRevisionRequest};
use jade_sync::application::store_access_token::{StoreAccessToken, StoreAccessTokenRequest};
use jade_sync::application::synchronize_vault::{SynchronizeVault, SynchronizeVaultRequest};
use jade_sync::domain::port::{Clock, CredentialStore, DeviceInfo};
use jade_sync::domain::value_object::RemoteUrl;
use jade_sync::infrastructure::Git2VersionControl;

use crate::errors::{ErrorCode, ErrorView};
use crate::view_models::{
    BacklinkView, BaseTableView, EntryView, FollowedLinkView, NotePropertiesView, PreferencesView,
    PropertySummaryView, PropertyView, RepositoryStatusView, ResolvedLinkView, RevisionView,
    SearchHitView, SyncReportView, VaultView,
};

const HISTORY_LIMIT: usize = 100;

pub struct Jade<C, K, D> {
    preferences: JsonFilePreferencesRepository,
    credentials: C,
    clock: K,
    device: D,
    vault: Mutex<Option<String>>,
    exclusive: Mutex<()>,
}

impl<C, K, D> Jade<C, K, D>
where
    C: CredentialStore,
    K: Clock,
    D: DeviceInfo,
{
    pub fn new(preferences_file: PathBuf, credentials: C, clock: K, device: D) -> Self {
        Self {
            preferences: JsonFilePreferencesRepository::new(preferences_file),
            credentials,
            clock,
            device,
            vault: Mutex::new(None),
            exclusive: Mutex::new(()),
        }
    }

    pub fn current_vault(&self) -> Option<String> {
        self.vault.lock().ok().and_then(|vault| vault.clone())
    }

    pub fn open_vault(&self, path: &str) -> Result<VaultView, ErrorView> {
        let info = OpenVault::new(FsNoteRepository).execute(OpenVaultRequest {
            vault_root: path.to_string(),
        })?;
        RememberVault::new(&self.preferences).execute(RememberVaultRequest {
            path: path.to_string(),
        })?;
        *self.vault.lock().map_err(poisoned)? = Some(path.to_string());
        Ok(info.into())
    }

    pub fn create_vault(&self, parent: &str, name: &str) -> Result<VaultView, ErrorView> {
        let info = CreateVault::new(FsNoteRepository).execute(CreateVaultRequest {
            parent: parent.to_string(),
            name: name.to_string(),
        })?;
        self.open_vault(&info.root.as_path().display().to_string())
    }

    pub fn list_entries(&self) -> Result<Vec<EntryView>, ErrorView> {
        let entries = ListVaultEntries::new(FsNoteRepository).execute(ListVaultEntriesRequest {
            vault_root: self.vault()?,
        })?;
        Ok(entries.into_iter().map(EntryView::from).collect())
    }

    pub fn read_note(&self, path: &str) -> Result<String, ErrorView> {
        Ok(ReadNote::new(FsNoteRepository).execute(ReadNoteRequest {
            vault_root: self.vault()?,
            path: path.to_string(),
        })?)
    }

    pub fn save_note(&self, path: &str, content: &str) -> Result<(), ErrorView> {
        let _exclusive = self.exclusive()?;
        Ok(SaveNote::new(FsNoteRepository).execute(SaveNoteRequest {
            vault_root: self.vault()?,
            path: path.to_string(),
            content: content.to_string(),
        })?)
    }

    pub fn create_note(&self, folder: Option<String>, title: &str) -> Result<String, ErrorView> {
        let _exclusive = self.exclusive()?;
        let path = CreateNote::new(FsNoteRepository).execute(CreateNoteRequest {
            vault_root: self.vault()?,
            folder,
            title: title.to_string(),
        })?;
        Ok(path.as_str().to_string())
    }

    pub fn create_folder(&self, path: &str) -> Result<String, ErrorView> {
        let _exclusive = self.exclusive()?;
        let path = CreateFolder::new(FsNoteRepository).execute(CreateFolderRequest {
            vault_root: self.vault()?,
            path: path.to_string(),
        })?;
        Ok(path.as_str().to_string())
    }

    pub fn rename_entry(&self, from: &str, to: &str) -> Result<String, ErrorView> {
        let _exclusive = self.exclusive()?;
        let path = RenameEntry::new(FsNoteRepository).execute(RenameEntryRequest {
            vault_root: self.vault()?,
            from: from.to_string(),
            to: to.to_string(),
        })?;
        Ok(path.as_str().to_string())
    }

    pub fn delete_entry(&self, path: &str) -> Result<(), ErrorView> {
        let _exclusive = self.exclusive()?;
        Ok(
            DeleteEntry::new(FsNoteRepository).execute(DeleteEntryRequest {
                vault_root: self.vault()?,
                path: path.to_string(),
            })?,
        )
    }

    pub fn search(&self, query: &str) -> Result<Vec<SearchHitView>, ErrorView> {
        let hits = SearchNotes::new(FsNoteRepository).execute(SearchNotesRequest {
            vault_root: self.vault()?,
            query: query.to_string(),
        })?;
        Ok(hits.into_iter().map(SearchHitView::from).collect())
    }

    pub fn backlinks(&self, path: &str) -> Result<Vec<BacklinkView>, ErrorView> {
        let backlinks = FindBacklinks::new(FsNoteRepository).execute(FindBacklinksRequest {
            vault_root: self.vault()?,
            path: path.to_string(),
        })?;
        Ok(backlinks.into_iter().map(BacklinkView::from).collect())
    }

    pub fn query_base(&self, path: &str, view: Option<String>) -> Result<BaseTableView, ErrorView> {
        let response =
            QueryBase::new(FsNoteRepository, YamlDataFormat).execute(QueryBaseRequest {
                vault_root: self.vault()?,
                path: path.to_string(),
                view,
            })?;
        Ok(BaseTableView::from(response))
    }

    pub fn read_properties(&self, content: String) -> Result<NotePropertiesView, ErrorView> {
        let properties = ReadProperties::new(YamlDataFormat).execute(ReadPropertiesRequest {
            content: content.clone(),
        })?;
        Ok(NotePropertiesView::new(properties, &content))
    }

    pub fn write_properties(
        &self,
        content: String,
        properties: Vec<PropertyView>,
    ) -> Result<String, ErrorView> {
        Ok(WriteProperties.execute(WritePropertiesRequest {
            content,
            properties: properties
                .into_iter()
                .map(|property| (property.key, property.value.into()))
                .collect(),
        })?)
    }

    pub fn list_properties(&self) -> Result<Vec<PropertySummaryView>, ErrorView> {
        let summaries = ListProperties::new(FsNoteRepository, YamlDataFormat).execute(
            ListPropertiesRequest {
                vault_root: self.vault()?,
            },
        )?;
        Ok(summaries
            .into_iter()
            .map(PropertySummaryView::from)
            .collect())
    }

    pub fn resolve_links(&self, targets: Vec<String>) -> Result<Vec<ResolvedLinkView>, ErrorView> {
        let links = ResolveLinks::new(FsNoteRepository).execute(ResolveLinksRequest {
            vault_root: self.vault()?,
            targets,
        })?;
        Ok(links.into_iter().map(ResolvedLinkView::from).collect())
    }

    pub fn follow_link(&self, target: &str) -> Result<FollowedLinkView, ErrorView> {
        let _exclusive = self.exclusive()?;
        let followed = FollowLink::new(FsNoteRepository).execute(FollowLinkRequest {
            vault_root: self.vault()?,
            target: target.to_string(),
        })?;
        Ok(followed.into())
    }

    pub fn repository_status(&self) -> Result<RepositoryStatusView, ErrorView> {
        let state =
            GetRepositoryStatus::new(Git2VersionControl).execute(GetRepositoryStatusRequest {
                working_directory: self.vault()?,
            })?;
        Ok(state.into())
    }

    pub fn synchronize(&self) -> Result<SyncReportView, ErrorView> {
        let _exclusive = self.exclusive()?;
        let report = SynchronizeVault::new(
            Git2VersionControl,
            &self.credentials,
            &self.clock,
            &self.device,
        )
        .execute(SynchronizeVaultRequest {
            working_directory: self.vault()?,
        })?;
        Ok(report.into())
    }

    pub fn init_repository(&self) -> Result<(), ErrorView> {
        let _exclusive = self.exclusive()?;
        Ok(
            InitRepository::new(Git2VersionControl).execute(InitRepositoryRequest {
                working_directory: self.vault()?,
            })?,
        )
    }

    pub fn connect_remote(&self, url: &str) -> Result<(), ErrorView> {
        Ok(
            ConnectRemote::new(Git2VersionControl).execute(ConnectRemoteRequest {
                working_directory: self.vault()?,
                url: url.to_string(),
            })?,
        )
    }

    pub fn clone_repository(&self, url: &str, parent: &str) -> Result<VaultView, ErrorView> {
        let name = RemoteUrl::parse(url)
            .map_err(|_| ErrorView::with(ErrorCode::InvalidUrl, "url", url))?
            .repository_name();
        let destination = Path::new(parent).join(name);
        let destination_text = destination.display().to_string();
        if destination.exists() {
            return Err(ErrorView::with(
                ErrorCode::AlreadyExists,
                "path",
                destination_text,
            ));
        }
        CloneRepository::new(Git2VersionControl, &self.credentials).execute(
            CloneRepositoryRequest {
                url: url.to_string(),
                working_directory: destination_text.clone(),
            },
        )?;
        self.open_vault(&destination_text)
    }

    pub fn note_history(&self, path: &str) -> Result<Vec<RevisionView>, ErrorView> {
        let revisions =
            ListNoteHistory::new(Git2VersionControl).execute(ListNoteHistoryRequest {
                working_directory: self.vault()?,
                path: path.to_string(),
                limit: HISTORY_LIMIT,
            })?;
        Ok(revisions.into_iter().map(RevisionView::from).collect())
    }

    pub fn note_revision(&self, path: &str, commit: &str) -> Result<String, ErrorView> {
        let bytes = ReadNoteRevision::new(Git2VersionControl).execute(ReadNoteRevisionRequest {
            working_directory: self.vault()?,
            path: path.to_string(),
            commit: commit.to_string(),
        })?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn store_access_token(&self, token: &str) -> Result<(), ErrorView> {
        Ok(
            StoreAccessToken::new(&self.credentials).execute(StoreAccessTokenRequest {
                token: token.to_string(),
            })?,
        )
    }

    pub fn has_access_token(&self) -> Result<bool, ErrorView> {
        Ok(CheckAccessToken::new(&self.credentials).execute(())?)
    }

    pub fn preferences(&self) -> Result<PreferencesView, ErrorView> {
        Ok(GetPreferences::new(&self.preferences).execute(())?.into())
    }

    pub fn update_preferences(
        &self,
        locale: Option<String>,
        theme: &str,
        auto_sync_minutes: u32,
    ) -> Result<PreferencesView, ErrorView> {
        let preferences =
            UpdatePreferences::new(&self.preferences).execute(UpdatePreferencesRequest {
                locale,
                theme: theme.to_string(),
                auto_sync_minutes,
            })?;
        Ok(preferences.into())
    }

    pub fn forget_vault(&self, path: &str) -> Result<PreferencesView, ErrorView> {
        let preferences = ForgetVault::new(&self.preferences).execute(ForgetVaultRequest {
            path: path.to_string(),
        })?;
        Ok(preferences.into())
    }

    /// Deletes a vault folder kept by the app in `container` (mobile app storage),
    /// closing it if it is open and forgetting it. A folder already gone is just forgotten.
    pub fn delete_vault(
        &self,
        path: &str,
        container: Option<&str>,
    ) -> Result<PreferencesView, ErrorView> {
        let container =
            container.ok_or_else(|| ErrorView::with(ErrorCode::InvalidPath, "path", path))?;
        let _exclusive = self.exclusive()?;
        match DeleteVault::new(FsNoteRepository).execute(DeleteVaultRequest {
            vault_root: path.to_string(),
            container: container.to_string(),
        }) {
            Ok(()) | Err(NotesError::VaultNotFound) => {}
            Err(error) => return Err(error.into()),
        }
        let mut open = self.vault.lock().map_err(poisoned)?;
        if open.as_deref() == Some(path) {
            *open = None;
        }
        drop(open);
        self.forget_vault(path)
    }

    fn vault(&self) -> Result<String, ErrorView> {
        self.vault
            .lock()
            .map_err(poisoned)?
            .clone()
            .ok_or_else(|| ErrorView::new(ErrorCode::VaultNotOpen))
    }

    fn exclusive(&self) -> Result<MutexGuard<'_, ()>, ErrorView> {
        self.exclusive.lock().map_err(poisoned)
    }
}

fn poisoned<T>(_: std::sync::PoisonError<T>) -> ErrorView {
    ErrorView::unexpected("lock poisoned")
}

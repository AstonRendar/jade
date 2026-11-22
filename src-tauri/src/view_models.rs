use jade_notes::application::follow_link::FollowedLink;
use jade_notes::application::open_vault::VaultInfo;
use jade_notes::application::resolve_links::ResolvedLink;
use jade_notes::domain::model::{EntryKind, VaultEntry};
use jade_notes::domain::service::{Backlink, SearchHit};
use jade_preferences::domain::model::Preferences;
use jade_sync::application::get_repository_status::RepositoryState;
use jade_sync::application::synchronize_vault::SyncReport;
use jade_sync::domain::port::Revision;
use serde::Serialize;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VaultView {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum EntryKindView {
    Folder,
    Note,
    File,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct EntryView {
    pub path: String,
    pub kind: EntryKindView,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SearchHitView {
    pub path: String,
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BacklinkView {
    pub source: String,
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResolvedLinkView {
    pub target: String,
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FollowedLinkView {
    pub path: String,
    pub created: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RepositoryStatusView {
    pub is_repository: bool,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub pending_changes: usize,
    pub ahead: usize,
    pub behind: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConflictCopyView {
    pub original: String,
    pub copy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SyncReportView {
    pub committed: bool,
    pub changed_paths: Vec<String>,
    pub conflict_copies: Vec<ConflictCopyView>,
    pub pushed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RevisionView {
    pub id: String,
    pub summary: String,
    pub author: String,
    pub epoch_seconds: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PreferencesView {
    pub locale: Option<String>,
    pub theme: String,
    pub auto_sync_minutes: u32,
    pub recent_vaults: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ChangedPathsView {
    pub paths: Vec<String>,
}

impl From<VaultInfo> for VaultView {
    fn from(info: VaultInfo) -> Self {
        Self {
            path: info.root.as_path().display().to_string(),
            name: info.name,
        }
    }
}

impl From<VaultEntry> for EntryView {
    fn from(entry: VaultEntry) -> Self {
        Self {
            path: entry.path.as_str().to_string(),
            kind: match entry.kind {
                EntryKind::Folder => EntryKindView::Folder,
                EntryKind::Note => EntryKindView::Note,
                EntryKind::File => EntryKindView::File,
            },
        }
    }
}

impl From<SearchHit> for SearchHitView {
    fn from(hit: SearchHit) -> Self {
        Self {
            path: hit.path.as_str().to_string(),
            line: hit.line,
            text: hit.text,
        }
    }
}

impl From<Backlink> for BacklinkView {
    fn from(backlink: Backlink) -> Self {
        Self {
            source: backlink.source.as_str().to_string(),
            line: backlink.line,
            text: backlink.text,
        }
    }
}

impl From<ResolvedLink> for ResolvedLinkView {
    fn from(link: ResolvedLink) -> Self {
        Self {
            path: link.note.map(|note| note.as_str().to_string()),
            target: link.target,
        }
    }
}

impl From<FollowedLink> for FollowedLinkView {
    fn from(link: FollowedLink) -> Self {
        Self {
            path: link.note.as_str().to_string(),
            created: link.created,
        }
    }
}

impl From<RepositoryState> for RepositoryStatusView {
    fn from(state: RepositoryState) -> Self {
        match state {
            RepositoryState::NotARepository => Self {
                is_repository: false,
                branch: None,
                remote: None,
                pending_changes: 0,
                ahead: 0,
                behind: 0,
            },
            RepositoryState::Ready(status) => Self {
                is_repository: true,
                branch: Some(status.branch.as_str().to_string()),
                remote: status.remote.map(|remote| remote.as_str().to_string()),
                pending_changes: status.pending_changes,
                ahead: status.ahead,
                behind: status.behind,
            },
        }
    }
}

impl From<SyncReport> for SyncReportView {
    fn from(report: SyncReport) -> Self {
        Self {
            committed: report.committed.is_some(),
            changed_paths: report
                .changed_paths
                .iter()
                .map(|path| path.as_str().to_string())
                .collect(),
            conflict_copies: report
                .conflict_copies
                .iter()
                .map(|conflict| ConflictCopyView {
                    original: conflict.original.as_str().to_string(),
                    copy: conflict.copy.as_str().to_string(),
                })
                .collect(),
            pushed: report.pushed,
        }
    }
}

impl From<Revision> for RevisionView {
    fn from(revision: Revision) -> Self {
        Self {
            id: revision.id.as_str().to_string(),
            summary: revision.summary,
            author: revision.author,
            epoch_seconds: revision.epoch_seconds,
        }
    }
}

impl From<Preferences> for PreferencesView {
    fn from(preferences: Preferences) -> Self {
        Self {
            locale: preferences
                .locale()
                .map(|locale| locale.as_str().to_string()),
            theme: preferences.theme().as_str().to_string(),
            auto_sync_minutes: preferences.auto_sync_minutes().value(),
            recent_vaults: preferences.recent_vaults().paths().to_vec(),
        }
    }
}

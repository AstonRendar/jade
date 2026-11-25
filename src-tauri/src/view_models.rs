use jade_notes::application::follow_link::FollowedLink;
use jade_notes::application::list_properties::PropertySummary;
use jade_notes::application::open_vault::VaultInfo;
use jade_notes::application::query_base::QueryBaseResponse;
use jade_notes::application::read_properties::NoteProperties;
use jade_notes::application::resolve_links::ResolvedLink;
use jade_notes::domain::model::{Cell, EntryKind, TableColumn, TableRow, VaultEntry};
use jade_notes::domain::service::{Backlink, SearchHit};
use jade_notes::domain::value_object::{Data, PropertyType};
use jade_preferences::domain::model::Preferences;
use jade_sync::application::get_repository_status::RepositoryState;
use jade_sync::application::list_branches::BranchList;
use jade_sync::application::synchronize_vault::SyncReport;
use jade_sync::domain::port::Revision;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub enum CellView {
    Null,
    Boolean {
        value: bool,
    },
    Number {
        value: f64,
    },
    Text {
        value: String,
    },
    Date {
        milliseconds: f64,
    },
    List {
        items: Vec<CellView>,
    },
    Link {
        target: String,
        label: Option<String>,
    },
    File {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BaseViewSummaryView {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BaseColumnView {
    pub key: String,
    pub display_name: Option<String>,
    pub width: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BaseRowView {
    pub path: String,
    pub cells: Vec<CellView>,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BaseTableView {
    pub views: Vec<BaseViewSummaryView>,
    pub view: u32,
    pub columns: Vec<BaseColumnView>,
    pub rows: Vec<BaseRowView>,
    pub total: u32,
}

impl From<Cell> for CellView {
    fn from(cell: Cell) -> Self {
        match cell {
            Cell::Null => Self::Null,
            Cell::Boolean(value) => Self::Boolean { value },
            Cell::Number(value) => Self::Number { value },
            Cell::Text(value) => Self::Text { value },
            Cell::Date(milliseconds) => Self::Date {
                milliseconds: milliseconds as f64,
            },
            Cell::List(items) => Self::List {
                items: items.into_iter().map(Self::from).collect(),
            },
            Cell::Link { target, label } => Self::Link { target, label },
            Cell::File(path) => Self::File {
                path: path.as_str().to_string(),
            },
        }
    }
}

impl From<TableColumn> for BaseColumnView {
    fn from(column: TableColumn) -> Self {
        Self {
            key: column.key,
            display_name: column.display_name,
            width: column.width,
        }
    }
}

impl From<TableRow> for BaseRowView {
    fn from(row: TableRow) -> Self {
        Self {
            path: row.path.as_str().to_string(),
            cells: row.cells.into_iter().map(CellView::from).collect(),
        }
    }
}

impl From<QueryBaseResponse> for BaseTableView {
    fn from(response: QueryBaseResponse) -> Self {
        Self {
            views: response
                .views
                .into_iter()
                .map(|view| BaseViewSummaryView {
                    name: view.name,
                    kind: view.kind,
                })
                .collect(),
            view: u32::try_from(response.view).unwrap_or(u32::MAX),
            columns: response
                .table
                .columns
                .into_iter()
                .map(BaseColumnView::from)
                .collect(),
            rows: response
                .table
                .rows
                .into_iter()
                .map(BaseRowView::from)
                .collect(),
            total: u32::try_from(response.table.total).unwrap_or(u32::MAX),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase", tag = "type")]
#[ts(export)]
pub enum PropertyValueView {
    Null,
    Boolean { value: bool },
    Number { value: f64 },
    Text { value: String },
    List { items: Vec<PropertyValueView> },
    Map { entries: Vec<PropertyView> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PropertyView {
    pub key: String,
    pub value: PropertyValueView,
}

#[derive(Debug, Clone, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NotePropertiesView {
    pub properties: Vec<PropertyView>,
    pub body_start: u32,
    pub valid: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PropertyTypeView {
    Text,
    List,
    Number,
    Checkbox,
    Date,
    DateTime,
    Tags,
    Aliases,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PropertySummaryView {
    pub name: String,
    pub kind: PropertyTypeView,
    pub count: u32,
    pub values: Vec<String>,
}

impl From<Data> for PropertyValueView {
    fn from(data: Data) -> Self {
        match data {
            Data::Null => Self::Null,
            Data::Boolean(value) => Self::Boolean { value },
            Data::Number(value) => Self::Number { value },
            Data::Text(value) => Self::Text { value },
            Data::List(items) => Self::List {
                items: items.into_iter().map(Self::from).collect(),
            },
            Data::Map(entries) => Self::Map {
                entries: entries.into_iter().map(PropertyView::from).collect(),
            },
        }
    }
}

impl From<PropertyValueView> for Data {
    fn from(value: PropertyValueView) -> Self {
        match value {
            PropertyValueView::Null => Self::Null,
            PropertyValueView::Boolean { value } => Self::Boolean(value),
            PropertyValueView::Number { value } => Self::Number(value),
            PropertyValueView::Text { value } => Self::Text(value),
            PropertyValueView::List { items } => {
                Self::List(items.into_iter().map(Self::from).collect())
            }
            PropertyValueView::Map { entries } => Self::Map(
                entries
                    .into_iter()
                    .map(|property| (property.key, Self::from(property.value)))
                    .collect(),
            ),
        }
    }
}

impl From<(String, Data)> for PropertyView {
    fn from((key, value): (String, Data)) -> Self {
        Self {
            key,
            value: PropertyValueView::from(value),
        }
    }
}

impl NotePropertiesView {
    pub fn new(properties: NoteProperties, content: &str) -> Self {
        let prefix = content.get(..properties.body_start).unwrap_or_default();
        Self {
            properties: properties
                .properties
                .into_iter()
                .map(PropertyView::from)
                .collect(),
            body_start: u32::try_from(prefix.encode_utf16().count()).unwrap_or(u32::MAX),
            valid: properties.valid,
        }
    }
}

impl From<PropertyType> for PropertyTypeView {
    fn from(kind: PropertyType) -> Self {
        match kind {
            PropertyType::Text => Self::Text,
            PropertyType::List => Self::List,
            PropertyType::Number => Self::Number,
            PropertyType::Checkbox => Self::Checkbox,
            PropertyType::Date => Self::Date,
            PropertyType::DateTime => Self::DateTime,
            PropertyType::Tags => Self::Tags,
            PropertyType::Aliases => Self::Aliases,
        }
    }
}

impl From<PropertySummary> for PropertySummaryView {
    fn from(summary: PropertySummary) -> Self {
        Self {
            name: summary.name,
            kind: PropertyTypeView::from(summary.kind),
            count: u32::try_from(summary.count).unwrap_or(u32::MAX),
            values: summary.values,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BranchListView {
    pub current: String,
    pub branches: Vec<String>,
}

impl From<BranchList> for BranchListView {
    fn from(list: BranchList) -> Self {
        Self {
            current: list.current.as_str().to_string(),
            branches: list
                .branches
                .iter()
                .map(|branch| branch.as_str().to_string())
                .collect(),
        }
    }
}

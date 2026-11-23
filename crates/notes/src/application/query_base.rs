use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::model::{Base, BaseTable, EntryKind, FileRecord, VaultEntry};
use crate::domain::port::{DataFormat, NoteRepository};
use crate::domain::service::{BaseQuery, FileCatalog, Frontmatter};
use crate::domain::value_object::{VaultPath, VaultRoot};

const BASE_EXTENSION: &str = ".base";

pub struct QueryBaseRequest {
    pub vault_root: String,
    pub path: String,
    pub view: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewSummary {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueryBaseResponse {
    pub views: Vec<ViewSummary>,
    pub view: usize,
    pub table: BaseTable,
}

pub struct QueryBase<R, F> {
    repository: R,
    format: F,
}

impl<R: NoteRepository, F: DataFormat> QueryBase<R, F> {
    pub fn new(repository: R, format: F) -> Self {
        Self { repository, format }
    }

    fn records(&self, root: &VaultRoot) -> Result<Vec<FileRecord>, NotesError> {
        self.repository
            .list(root)?
            .into_iter()
            .filter(|entry| entry.kind != EntryKind::Folder)
            .map(|entry| self.record(root, entry))
            .collect()
    }

    fn record(&self, root: &VaultRoot, entry: VaultEntry) -> Result<FileRecord, NotesError> {
        let metadata = self.repository.metadata(root, &entry.path)?;
        if entry.kind != EntryKind::Note {
            return Ok(FileCatalog.file(entry.path, metadata));
        }
        let content = self.repository.read_text(root, &entry.path)?;
        let properties = Frontmatter::split(&content)
            .yaml
            .and_then(|yaml| self.format.parse(yaml).ok());
        Ok(FileCatalog.note(entry.path, metadata, properties.as_ref(), &content))
    }
}

impl<R: NoteRepository, F: DataFormat> UseCase for QueryBase<R, F> {
    type Request = QueryBaseRequest;
    type Response = QueryBaseResponse;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let path = VaultPath::parse(&request.path)?;
        if !path.file_name().to_lowercase().ends_with(BASE_EXTENSION) {
            return Err(NotesError::InvalidPath(request.path));
        }
        let text = self.repository.read_text(&root, &path)?;
        let data = self
            .format
            .parse(&text)
            .map_err(|error| NotesError::InvalidBase(error.0))?;
        let base =
            Base::from_data(&data).map_err(|error| NotesError::InvalidBase(error.to_string()))?;
        let index = request
            .view
            .and_then(|name| base.views.iter().position(|view| view.name == name))
            .unwrap_or(0);
        let records = self.records(&root)?;
        let this = records
            .iter()
            .find(|record| *record.path() == path)
            .cloned()
            .ok_or(NotesError::NotFound(path))?;
        let table = BaseQuery.run(&base, &base.views[index], &records, &this);
        Ok(QueryBaseResponse {
            views: base
                .views
                .iter()
                .map(|view| ViewSummary {
                    name: view.name.clone(),
                    kind: view.kind.clone(),
                })
                .collect(),
            view: index,
            table,
        })
    }
}

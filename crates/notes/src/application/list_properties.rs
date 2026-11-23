use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::model::EntryKind;
use crate::domain::port::{DataFormat, NoteRepository};
use crate::domain::service::Frontmatter;
use crate::domain::value_object::{Data, PropertyType, VaultPath, VaultRoot};

const TYPES_FILE: &str = ".obsidian/types.json";
const MAX_SUGGESTIONS: usize = 200;

pub struct ListPropertiesRequest {
    pub vault_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertySummary {
    pub name: String,
    pub kind: PropertyType,
    pub count: usize,
    pub values: Vec<String>,
}

pub struct ListProperties<R, F> {
    repository: R,
    format: F,
}

impl<R: NoteRepository, F: DataFormat> ListProperties<R, F> {
    pub fn new(repository: R, format: F) -> Self {
        Self { repository, format }
    }

    fn declared_types(&self, root: &VaultRoot) -> Vec<(String, PropertyType)> {
        let Ok(path) = VaultPath::parse(TYPES_FILE) else {
            return Vec::new();
        };
        let Ok(text) = self.repository.read_text(root, &path) else {
            return Vec::new();
        };
        self.format
            .parse(&text)
            .ok()
            .and_then(|data| data.get("types").and_then(Data::as_map).map(<[_]>::to_vec))
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(name, kind)| {
                kind.as_text()
                    .and_then(PropertyType::from_name)
                    .map(|kind| (name, kind))
            })
            .collect()
    }
}

impl<R: NoteRepository, F: DataFormat> UseCase for ListProperties<R, F> {
    type Request = ListPropertiesRequest;
    type Response = Vec<PropertySummary>;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let declared = self.declared_types(&root);
        let mut summaries: Vec<(PropertySummary, bool)> = Vec::new();
        for entry in self.repository.list(&root)? {
            if entry.kind != EntryKind::Note {
                continue;
            }
            let content = self.repository.read_text(&root, &entry.path)?;
            let Some(Ok(Data::Map(properties))) = Frontmatter::split(&content)
                .yaml
                .map(|yaml| self.format.parse(yaml))
            else {
                continue;
            };
            for (name, value) in properties {
                collect(&mut summaries, &declared, name, &value);
            }
        }
        let mut summaries: Vec<PropertySummary> = summaries
            .into_iter()
            .map(|(mut summary, _)| {
                summary.values.sort();
                summary.values.truncate(MAX_SUGGESTIONS);
                summary
            })
            .collect();
        summaries.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(summaries)
    }
}

fn collect(
    summaries: &mut Vec<(PropertySummary, bool)>,
    declared: &[(String, PropertyType)],
    name: String,
    value: &Data,
) {
    let declared_kind = declared
        .iter()
        .find(|(declared, _)| *declared == name)
        .map(|(_, kind)| *kind);
    let inferred = PropertyType::infer(&name, value);
    let index = match summaries
        .iter()
        .position(|(summary, _)| summary.name == name)
    {
        Some(index) => index,
        None => {
            summaries.push((
                PropertySummary {
                    name,
                    kind: declared_kind.or(inferred).unwrap_or(PropertyType::Text),
                    count: 0,
                    values: Vec::new(),
                },
                declared_kind.is_some() || inferred.is_some(),
            ));
            summaries.len() - 1
        }
    };
    let (summary, settled) = &mut summaries[index];
    summary.count += 1;
    if !*settled {
        if let Some(kind) = inferred {
            summary.kind = kind;
            *settled = true;
        }
    }
    let texts: Vec<&str> = match value {
        Data::Text(text) => vec![text.as_str()],
        Data::List(items) => items.iter().filter_map(Data::as_text).collect(),
        _ => Vec::new(),
    };
    for text in texts {
        if !text.is_empty() && !summary.values.iter().any(|known| known == text) {
            summary.values.push(text.to_string());
        }
    }
}

use jade_shared::application::use_case::UseCase;

use super::error::{vault_root, NotesError};
use crate::domain::port::{DataFormat, NoteRepository, NoteRepositoryError};
use crate::domain::service::JsonWriter;
use crate::domain::value_object::{Data, PropertyType, VaultPath};

const TYPES_FILE: &str = ".obsidian/types.json";
const TYPES_KEY: &str = "types";

pub struct SetPropertyTypeRequest {
    pub vault_root: String,
    pub name: String,
    pub kind: String,
}

pub struct SetPropertyType<R, F> {
    repository: R,
    format: F,
}

impl<R: NoteRepository, F: DataFormat> SetPropertyType<R, F> {
    pub fn new(repository: R, format: F) -> Self {
        Self { repository, format }
    }
}

impl<R: NoteRepository, F: DataFormat> UseCase for SetPropertyType<R, F> {
    type Request = SetPropertyTypeRequest;
    type Response = ();
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let root = vault_root(&request.vault_root)?;
        let kind = PropertyType::from_name(&request.kind)
            .filter(|kind| kind.is_assignable())
            .ok_or_else(|| NotesError::InvalidProperty(request.kind.clone()))?;
        let name = request.name.trim();
        if name.is_empty() || name == "tags" || name == "aliases" {
            return Err(NotesError::InvalidProperty(request.name));
        }
        let path = VaultPath::parse(TYPES_FILE)?;
        let mut document = match self.repository.read_text(&root, &path) {
            Ok(text) => match self.format.parse(&text) {
                Ok(Data::Map(entries)) => entries,
                Ok(Data::Null) => Vec::new(),
                _ => return Err(NotesError::InvalidProperty(TYPES_FILE.to_string())),
            },
            Err(NoteRepositoryError::NotFound(_)) => Vec::new(),
            Err(error) => return Err(error.into()),
        };
        if !document.iter().any(|(key, _)| key == TYPES_KEY) {
            document.push((TYPES_KEY.to_string(), Data::Map(Vec::new())));
        }
        let Some((_, Data::Map(entries))) = document.iter_mut().find(|(key, _)| key == TYPES_KEY)
        else {
            return Err(NotesError::InvalidProperty(TYPES_FILE.to_string()));
        };
        let value = Data::Text(kind.name().to_string());
        match entries.iter_mut().find(|(key, _)| key == name) {
            Some((_, existing)) => *existing = value,
            None => entries.push((name.to_string(), value)),
        }
        let json = JsonWriter.write(&Data::Map(document));
        Ok(self.repository.write_text(&root, &path, &json)?)
    }
}

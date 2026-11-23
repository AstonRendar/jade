use jade_shared::application::use_case::UseCase;

use super::error::NotesError;
use crate::domain::service::FrontmatterWriter;
use crate::domain::value_object::Data;

pub struct WritePropertiesRequest {
    pub content: String,
    pub properties: Vec<(String, Data)>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct WriteProperties;

impl UseCase for WriteProperties {
    type Request = WritePropertiesRequest;
    type Response = String;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let mut seen: Vec<&str> = Vec::new();
        for (key, _) in &request.properties {
            if key.trim().is_empty() || seen.contains(&key.as_str()) {
                return Err(NotesError::InvalidProperty(key.clone()));
            }
            seen.push(key);
        }
        Ok(FrontmatterWriter.write(&request.content, &request.properties))
    }
}

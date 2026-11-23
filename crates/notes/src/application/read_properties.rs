use jade_shared::application::use_case::UseCase;

use super::error::NotesError;
use crate::domain::port::DataFormat;
use crate::domain::service::Frontmatter;
use crate::domain::value_object::Data;

pub struct ReadPropertiesRequest {
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoteProperties {
    pub properties: Vec<(String, Data)>,
    pub body_start: usize,
    pub valid: bool,
}

pub struct ReadProperties<F> {
    format: F,
}

impl<F: DataFormat> ReadProperties<F> {
    pub fn new(format: F) -> Self {
        Self { format }
    }
}

impl<F: DataFormat> UseCase for ReadProperties<F> {
    type Request = ReadPropertiesRequest;
    type Response = NoteProperties;
    type Error = NotesError;

    fn execute(&self, request: Self::Request) -> Result<Self::Response, Self::Error> {
        let split = Frontmatter::split(&request.content);
        let Some(yaml) = split.yaml else {
            return Ok(NoteProperties {
                properties: Vec::new(),
                body_start: 0,
                valid: true,
            });
        };
        let body_start = request.content.len() - split.body.len();
        Ok(match self.format.parse(yaml) {
            Ok(Data::Map(properties)) => NoteProperties {
                properties,
                body_start,
                valid: true,
            },
            Ok(Data::Null) => NoteProperties {
                properties: Vec::new(),
                body_start,
                valid: true,
            },
            _ => NoteProperties {
                properties: Vec::new(),
                body_start: 0,
                valid: false,
            },
        })
    }
}

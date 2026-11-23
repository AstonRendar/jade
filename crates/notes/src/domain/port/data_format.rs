use crate::domain::value_object::Data;

pub trait DataFormat {
    fn parse(&self, text: &str) -> Result<Data, DataFormatError>;
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("malformed document: {0}")]
pub struct DataFormatError(pub String);

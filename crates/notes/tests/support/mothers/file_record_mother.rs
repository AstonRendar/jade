use jade_notes::domain::model::{FileMetadata, FileRecord};
use jade_notes::domain::value_object::{Data, VaultPath};

pub const MODIFIED_MS: i64 = 1_767_225_600_000;

pub struct FileRecordMother;

impl FileRecordMother {
    pub fn metadata() -> FileMetadata {
        FileMetadata {
            size: 120,
            created_ms: MODIFIED_MS - 86_400_000,
            modified_ms: MODIFIED_MS,
        }
    }

    pub fn file(path: &str) -> FileRecord {
        FileRecord::new(
            VaultPath::parse(path).expect("valid path"),
            Self::metadata(),
        )
    }

    pub fn note(
        path: &str,
        properties: Vec<(&str, Data)>,
        tags: &[&str],
        links: &[&str],
    ) -> FileRecord {
        Self::file(path).with_contents(
            properties
                .into_iter()
                .map(|(key, value)| (key.to_string(), value))
                .collect(),
            tags.iter().map(|tag| tag.to_string()).collect(),
            links.iter().map(|link| link.to_string()).collect(),
        )
    }

    pub fn text(value: &str) -> Data {
        Data::Text(value.to_string())
    }

    pub fn texts(values: &[&str]) -> Data {
        Data::List(values.iter().map(|value| Self::text(value)).collect())
    }
}

use jade_notes::application::error::NotesError;
use jade_notes::application::list_properties::{
    ListProperties, ListPropertiesRequest, PropertySummary,
};
use jade_notes::application::read_properties::{
    NoteProperties, ReadProperties, ReadPropertiesRequest,
};
use jade_notes::application::write_properties::{WriteProperties, WritePropertiesRequest};
use jade_notes::domain::model::{EntryKind, VaultEntry};
use jade_notes::domain::port::{DataFormatError, NoteRepositoryError};
use jade_notes::domain::value_object::{Data, PropertyType, VaultPath};
use jade_shared::application::use_case::UseCase;

use crate::support::mocks::{MockDataFormat, MockNoteRepository};
use crate::support::mothers::{FileRecordMother, VAULT};

fn text(value: &str) -> Data {
    Data::Text(value.to_string())
}

#[test]
fn reads_the_properties_and_where_the_body_starts() {
    // Arrange
    let content = "---\ntipo: lore\n---\nbody";

    // Act
    let read =
        ReadProperties::new(given_the_documents_are_understood()).execute(ReadPropertiesRequest {
            content: content.to_string(),
        });

    // Assert
    assert_eq!(
        read,
        Ok(NoteProperties {
            properties: vec![("tipo".to_string(), text("lore"))],
            body_start: 19,
            valid: true,
        })
    );
}

#[test]
fn reads_notes_without_or_with_empty_frontmatter() {
    // Arrange
    let contents = ["just text", "---\n---\nbody"];

    // Act
    let read: Vec<_> = contents
        .iter()
        .map(|content| {
            ReadProperties::new(given_the_documents_are_understood())
                .execute(ReadPropertiesRequest {
                    content: content.to_string(),
                })
                .expect("properties are read")
        })
        .collect();

    // Assert
    assert_eq!((read[0].body_start, read[0].valid), (0, true));
    assert_eq!(
        (read[1].body_start, read[1].valid, read[1].properties.len()),
        (8, true, 0)
    );
}

#[test]
fn marks_malformed_frontmatter_as_invalid_and_keeps_it_in_the_body() {
    // Arrange
    let contents = ["---\n: [\n---\nbody", "---\n- a\n---\nbody"];

    // Act
    let read: Vec<_> = contents
        .iter()
        .map(|content| {
            ReadProperties::new(given_the_documents_are_understood())
                .execute(ReadPropertiesRequest {
                    content: content.to_string(),
                })
                .expect("properties are read")
        })
        .collect();

    // Assert
    for properties in read {
        assert_eq!(
            properties,
            NoteProperties {
                properties: Vec::new(),
                body_start: 0,
                valid: false
            }
        );
    }
}

#[test]
fn writes_properties_into_the_content() {
    // Arrange
    let request = WritePropertiesRequest {
        content: "body".to_string(),
        properties: vec![("tipo".to_string(), text("lore"))],
    };

    // Act
    let written = WriteProperties.execute(request);

    // Assert
    assert_eq!(written, Ok("---\ntipo: lore\n---\nbody".to_string()));
}

#[test]
fn refuses_blank_or_repeated_property_names() {
    // Arrange
    let lists = [
        vec![(" ".to_string(), Data::Null)],
        vec![("a".to_string(), Data::Null), ("a".to_string(), Data::Null)],
    ];

    // Act
    let results: Vec<_> = lists
        .into_iter()
        .map(|properties| {
            WriteProperties.execute(WritePropertiesRequest {
                content: String::new(),
                properties,
            })
        })
        .collect();

    // Assert
    assert_eq!(
        results,
        vec![
            Err(NotesError::InvalidProperty(" ".to_string())),
            Err(NotesError::InvalidProperty("a".to_string())),
        ]
    );
}

#[test]
fn lists_every_property_with_its_type_count_and_values() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_a_vault_with_typed_properties(&mut repository);

    // Act
    let summaries = ListProperties::new(repository, given_the_documents_are_understood()).execute(
        ListPropertiesRequest {
            vault_root: VAULT.to_string(),
        },
    );

    // Assert
    assert_eq!(
        summaries,
        Ok(vec![
            summary("anyo", PropertyType::Number, 2, &["1024"]),
            summary("tags", PropertyType::Tags, 2, &["moc", "saga"]),
            summary("tardío", PropertyType::Checkbox, 2, &[]),
            summary("tipo", PropertyType::Text, 2, &["lore", "personaje"]),
        ])
    );
}

#[test]
fn ignores_a_missing_or_malformed_types_file() {
    // Arrange
    let mut missing = MockNoteRepository::new();
    missing
        .expect_read_text()
        .returning(|_, path| Err(NoteRepositoryError::NotFound(path.clone())));
    missing.expect_list().returning(|_| Ok(Vec::new()));
    let mut malformed = MockNoteRepository::new();
    malformed
        .expect_read_text()
        .returning(|_, _| Ok("{".to_string()));
    malformed.expect_list().returning(|_| Ok(Vec::new()));

    // Act
    let results = [
        ListProperties::new(missing, given_the_documents_are_understood()).execute(
            ListPropertiesRequest {
                vault_root: VAULT.to_string(),
            },
        ),
        ListProperties::new(malformed, given_the_documents_are_understood()).execute(
            ListPropertiesRequest {
                vault_root: VAULT.to_string(),
            },
        ),
    ];

    // Assert
    assert_eq!(results, [Ok(Vec::new()), Ok(Vec::new())]);
}

#[test]
fn reports_an_invalid_vault_and_unreadable_notes() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_read_text()
        .returning(|_, path| match path.as_str() {
            ".obsidian/types.json" => Err(NoteRepositoryError::NotFound(path.clone())),
            _ => Err(NoteRepositoryError::Io("denied".to_string())),
        });
    repository
        .expect_list()
        .returning(|_| Ok(vec![entry("a.md", EntryKind::Note)]));

    // Act
    let invalid = ListProperties::new(MockNoteRepository::new(), MockDataFormat::new()).execute(
        ListPropertiesRequest {
            vault_root: "relative".to_string(),
        },
    );
    let unreadable = ListProperties::new(repository, given_the_documents_are_understood()).execute(
        ListPropertiesRequest {
            vault_root: VAULT.to_string(),
        },
    );

    // Assert
    assert!(matches!(invalid, Err(NotesError::InvalidVaultRoot(_))));
    assert_eq!(unreadable, Err(NotesError::Storage("denied".to_string())));
}

fn given_the_documents_are_understood() -> MockDataFormat {
    let mut format = MockDataFormat::new();
    format.expect_parse().returning(|yaml| match yaml {
        "tipo: lore\n" => Ok(Data::Map(vec![("tipo".to_string(), text("lore"))])),
        "" => Ok(Data::Null),
        "- a\n" => Ok(Data::List(vec![text("a")])),
        "types" => Ok(Data::Map(vec![(
            "types".to_string(),
            Data::Map(vec![
                ("anyo".to_string(), text("number")),
                ("tardío".to_string(), text("checkbox")),
                ("raro".to_string(), text("select")),
                ("otro".to_string(), Data::Number(1.0)),
            ]),
        )])),
        "a\n" => Ok(Data::Map(vec![
            ("tipo".to_string(), text("personaje")),
            ("anyo".to_string(), text("1024")),
            (
                "tags".to_string(),
                FileRecordMother::texts(&["saga", "moc"]),
            ),
            ("tardío".to_string(), Data::Null),
        ])),
        "b\n" => Ok(Data::Map(vec![
            ("tipo".to_string(), text("lore")),
            ("anyo".to_string(), text("1024")),
            ("tags".to_string(), FileRecordMother::texts(&["saga", ""])),
            ("tardío".to_string(), Data::Boolean(true)),
        ])),
        _ => Err(DataFormatError("malformed".to_string())),
    });
    format
}

fn given_a_vault_with_typed_properties(repository: &mut MockNoteRepository) {
    repository.expect_list().returning(|_| {
        Ok(vec![
            entry("folder", EntryKind::Folder),
            entry("a.md", EntryKind::Note),
            entry("image.png", EntryKind::File),
            entry("b.md", EntryKind::Note),
            entry("plain.md", EntryKind::Note),
            entry("broken.md", EntryKind::Note),
        ])
    });
    repository.expect_read_text().returning(|_, path| {
        Ok(match path.as_str() {
            ".obsidian/types.json" => "types".to_string(),
            "a.md" => "---\na\n---\n".to_string(),
            "b.md" => "---\nb\n---\n".to_string(),
            "broken.md" => "---\n: [\n---\n".to_string(),
            _ => "plain".to_string(),
        })
    });
}

fn summary(name: &str, kind: PropertyType, count: usize, values: &[&str]) -> PropertySummary {
    PropertySummary {
        name: name.to_string(),
        kind,
        count,
        values: values.iter().map(|value| value.to_string()).collect(),
    }
}

fn entry(path: &str, kind: EntryKind) -> VaultEntry {
    VaultEntry {
        path: VaultPath::parse(path).expect("valid path"),
        kind,
    }
}

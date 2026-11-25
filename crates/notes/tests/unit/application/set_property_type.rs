use std::sync::{Arc, Mutex};

use jade_notes::application::error::NotesError;
use jade_notes::application::set_property_type::{SetPropertyType, SetPropertyTypeRequest};
use jade_notes::domain::port::{DataFormatError, NoteRepositoryError};
use jade_notes::domain::value_object::Data;
use jade_shared::application::use_case::UseCase;

use crate::support::mocks::{MockDataFormat, MockNoteRepository};
use crate::support::mothers::VAULT;

#[test]
fn adds_the_type_to_an_existing_types_file_keeping_the_rest() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_types_file_contains(&mut repository, "existing");
    let written = expect_the_types_file_is_written(&mut repository);

    // Act
    let result = SetPropertyType::new(repository, given_the_documents_are_understood())
        .execute(request("anyo", "number"));

    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(
        written.lock().expect("lock").as_str(),
        "{\n  \"other\": true,\n  \"types\": {\n    \"tipo\": \"text\",\n    \"anyo\": \"number\"\n  }\n}\n"
    );
}

#[test]
fn replaces_the_type_of_a_known_property() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_types_file_contains(&mut repository, "existing");
    let written = expect_the_types_file_is_written(&mut repository);

    // Act
    let result = SetPropertyType::new(repository, given_the_documents_are_understood())
        .execute(request("tipo", "multitext"));

    // Assert
    assert_eq!(result, Ok(()));
    assert!(written
        .lock()
        .expect("lock")
        .contains("\"tipo\": \"multitext\""));
}

#[test]
fn creates_the_types_file_when_missing_or_empty() {
    // Arrange
    let mut missing = MockNoteRepository::new();
    missing
        .expect_read_text()
        .returning(|_, path| Err(NoteRepositoryError::NotFound(path.clone())));
    let from_missing = expect_the_types_file_is_written(&mut missing);
    let mut empty = MockNoteRepository::new();
    given_the_types_file_contains(&mut empty, "");
    let from_empty = expect_the_types_file_is_written(&mut empty);

    // Act
    let results = [
        SetPropertyType::new(missing, given_the_documents_are_understood())
            .execute(request("fecha", "date")),
        SetPropertyType::new(empty, given_the_documents_are_understood())
            .execute(request("fecha", "date")),
    ];

    // Assert
    assert_eq!(results, [Ok(()), Ok(())]);
    let expected = "{\n  \"types\": {\n    \"fecha\": \"date\"\n  }\n}\n";
    assert_eq!(from_missing.lock().expect("lock").as_str(), expected);
    assert_eq!(from_empty.lock().expect("lock").as_str(), expected);
}

#[test]
fn refuses_unknown_or_fixed_types_and_blank_or_fixed_names() {
    // Arrange
    let requests = [
        request("anyo", "select"),
        request("anyo", "tags"),
        request(" ", "text"),
        request("tags", "text"),
        request("aliases", "multitext"),
    ];

    // Act
    let results: Vec<_> = requests
        .into_iter()
        .map(|request| {
            SetPropertyType::new(MockNoteRepository::new(), MockDataFormat::new()).execute(request)
        })
        .collect();

    // Assert
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(NotesError::InvalidProperty(_)))),
        "{results:?}"
    );
}

#[test]
fn leaves_a_malformed_types_file_untouched() {
    // Arrange
    let documents = ["malformed", "list", "types-list"];

    // Act
    let results: Vec<_> = documents
        .iter()
        .map(|document| {
            let mut repository = MockNoteRepository::new();
            given_the_types_file_contains(&mut repository, document);
            SetPropertyType::new(repository, given_the_documents_are_understood())
                .execute(request("anyo", "number"))
        })
        .collect();

    // Assert
    assert!(results.iter().all(|result| *result
        == Err(NotesError::InvalidProperty(
            ".obsidian/types.json".to_string()
        ))));
}

#[test]
fn reports_storage_failures() {
    // Arrange
    let mut unreadable = MockNoteRepository::new();
    unreadable
        .expect_read_text()
        .returning(|_, _| Err(NoteRepositoryError::Io("denied".to_string())));

    // Act
    let result =
        SetPropertyType::new(unreadable, MockDataFormat::new()).execute(request("anyo", "number"));
    let invalid = SetPropertyType::new(MockNoteRepository::new(), MockDataFormat::new()).execute(
        SetPropertyTypeRequest {
            vault_root: "relative".to_string(),
            name: "anyo".to_string(),
            kind: "number".to_string(),
        },
    );

    // Assert
    assert_eq!(result, Err(NotesError::Storage("denied".to_string())));
    assert!(matches!(invalid, Err(NotesError::InvalidVaultRoot(_))));
}

fn request(name: &str, kind: &str) -> SetPropertyTypeRequest {
    SetPropertyTypeRequest {
        vault_root: VAULT.to_string(),
        name: name.to_string(),
        kind: kind.to_string(),
    }
}

fn given_the_types_file_contains(repository: &mut MockNoteRepository, text: &'static str) {
    repository
        .expect_read_text()
        .returning(move |_, _| Ok(text.to_string()));
}

fn expect_the_types_file_is_written(repository: &mut MockNoteRepository) -> Arc<Mutex<String>> {
    let written = Arc::new(Mutex::new(String::new()));
    let sink = Arc::clone(&written);
    repository
        .expect_write_text()
        .withf(|_, path, _| path.as_str() == ".obsidian/types.json")
        .times(1)
        .returning(move |_, _, content| {
            *sink.lock().expect("lock") = content.to_string();
            Ok(())
        });
    written
}

fn given_the_documents_are_understood() -> MockDataFormat {
    let mut format = MockDataFormat::new();
    format.expect_parse().returning(|text| match text {
        "existing" => Ok(Data::Map(vec![
            ("other".to_string(), Data::Boolean(true)),
            (
                "types".to_string(),
                Data::Map(vec![("tipo".to_string(), Data::Text("text".to_string()))]),
            ),
        ])),
        "" => Ok(Data::Null),
        "list" => Ok(Data::List(Vec::new())),
        "types-list" => Ok(Data::Map(vec![(
            "types".to_string(),
            Data::List(Vec::new()),
        )])),
        _ => Err(DataFormatError("malformed".to_string())),
    });
    format
}

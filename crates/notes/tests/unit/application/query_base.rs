use jade_notes::application::error::NotesError;
use jade_notes::application::query_base::{QueryBase, QueryBaseRequest, ViewSummary};
use jade_notes::domain::model::{Cell, EntryKind, VaultEntry};
use jade_notes::domain::port::{DataFormatError, NoteRepositoryError};
use jade_notes::domain::value_object::{Data, VaultPath};
use jade_shared::application::use_case::UseCase;

use crate::support::mocks::{MockDataFormat, MockNoteRepository};
use crate::support::mothers::{FileRecordMother, NoteMother, VAULT};

const BASE: &str = "views:\n  - name: First\n  - name: Second\n";

#[test]
fn returns_the_rows_of_the_requested_view() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_vault_contains(&mut repository);
    let format = given_the_documents_are_understood();

    // Act
    let response =
        QueryBase::new(repository, format).execute(request("Index.base", Some("Second")));

    // Assert
    let response = response.expect("the base is queried");
    assert_eq!(
        response.views,
        vec![
            ViewSummary {
                name: "First".to_string(),
                kind: "table".to_string()
            },
            ViewSummary {
                name: "Second".to_string(),
                kind: "table".to_string()
            },
        ]
    );
    assert_eq!(response.view, 1);
    let rows: Vec<_> = response
        .table
        .rows
        .iter()
        .map(|row| (row.path.as_str(), row.cells.clone()))
        .collect();
    assert_eq!(
        rows,
        vec![
            ("Ana.md", vec![Cell::Text("personaje".to_string())]),
            ("Index.base", vec![Cell::Null]),
            ("image.png", vec![Cell::Null]),
            ("Malformed.md", vec![Cell::Null]),
        ]
    );
}

#[test]
fn falls_back_to_the_first_view_when_the_name_is_unknown() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_vault_contains(&mut repository);

    // Act
    let response = QueryBase::new(repository, given_the_documents_are_understood())
        .execute(request("Index.base", Some("Missing")));

    // Assert
    assert_eq!(response.map(|response| response.view), Ok(0));
}

#[test]
fn rejects_paths_that_are_not_bases() {
    // Arrange
    let paths = ["Ana.md", "../Index.base"];

    // Act
    let results: Vec<_> = paths
        .iter()
        .map(|path| {
            QueryBase::new(MockNoteRepository::new(), MockDataFormat::new())
                .execute(request(path, None))
        })
        .collect();

    // Assert
    assert!(results
        .iter()
        .all(|result| matches!(result, Err(NotesError::InvalidPath(_)))));
}

#[test]
fn reports_malformed_base_documents() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_read_text()
        .returning(|_, _| Ok("views: [".to_string()));
    let mut format = MockDataFormat::new();
    format
        .expect_parse()
        .returning(|_| Err(DataFormatError("unclosed".to_string())));
    let mut wrong = MockNoteRepository::new();
    wrong
        .expect_read_text()
        .returning(|_, _| Ok("- a".to_string()));
    let mut list = MockDataFormat::new();
    list.expect_parse()
        .returning(|_| Ok(Data::List(Vec::new())));

    // Act
    let malformed = QueryBase::new(repository, format).execute(request("Index.base", None));
    let invalid = QueryBase::new(wrong, list).execute(request("Index.base", None));

    // Assert
    assert_eq!(
        malformed,
        Err(NotesError::InvalidBase("unclosed".to_string()))
    );
    assert!(matches!(invalid, Err(NotesError::InvalidBase(_))));
}

#[test]
fn reports_a_missing_base() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_read_text()
        .returning(|_, path| Err(NoteRepositoryError::NotFound(path.clone())));

    // Act
    let result =
        QueryBase::new(repository, MockDataFormat::new()).execute(request("Index.base", None));

    // Assert
    assert_eq!(
        result,
        Err(NotesError::NotFound(NoteMother::vault_path("Index.base")))
    );
}

#[test]
fn reports_a_base_that_is_not_listed_in_the_vault() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_read_text()
        .returning(|_, _| Ok(BASE.to_string()));
    repository.expect_list().returning(|_| Ok(Vec::new()));

    // Act
    let result = QueryBase::new(repository, given_the_documents_are_understood())
        .execute(request("Index.base", None));

    // Assert
    assert_eq!(
        result,
        Err(NotesError::NotFound(NoteMother::vault_path("Index.base")))
    );
}

fn request(path: &str, view: Option<&str>) -> QueryBaseRequest {
    QueryBaseRequest {
        vault_root: VAULT.to_string(),
        path: path.to_string(),
        view: view.map(str::to_string),
    }
}

fn given_the_vault_contains(repository: &mut MockNoteRepository) {
    repository.expect_list().returning(|_| {
        Ok(vec![
            entry("Characters", EntryKind::Folder),
            entry("Ana.md", EntryKind::Note),
            entry("Index.base", EntryKind::File),
            entry("image.png", EntryKind::File),
            entry("Malformed.md", EntryKind::Note),
        ])
    });
    repository
        .expect_metadata()
        .returning(|_, _| Ok(FileRecordMother::metadata()));
    repository.expect_read_text().returning(|_, path| {
        Ok(match path.as_str() {
            "Index.base" => BASE.to_string(),
            "Ana.md" => "---\ntipo: personaje\n---\nbody".to_string(),
            _ => "---\n: [\n---\n".to_string(),
        })
    });
}

fn given_the_documents_are_understood() -> MockDataFormat {
    let mut format = MockDataFormat::new();
    format.expect_parse().returning(|text| match text {
        BASE => Ok(Data::Map(vec![(
            "views".to_string(),
            Data::List(vec![view("First", "file.name"), view("Second", "tipo")]),
        )])),
        "tipo: personaje\n" => Ok(Data::Map(vec![(
            "tipo".to_string(),
            FileRecordMother::text("personaje"),
        )])),
        _ => Err(DataFormatError("malformed".to_string())),
    });
    format
}

fn view(name: &str, column: &str) -> Data {
    Data::Map(vec![
        ("name".to_string(), FileRecordMother::text(name)),
        ("order".to_string(), FileRecordMother::texts(&[column])),
    ])
}

fn entry(path: &str, kind: EntryKind) -> VaultEntry {
    VaultEntry {
        path: VaultPath::parse(path).expect("valid path"),
        kind,
    }
}

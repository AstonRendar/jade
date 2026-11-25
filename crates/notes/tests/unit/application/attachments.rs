use std::sync::{Arc, Mutex};

use jade_notes::application::error::NotesError;
use jade_notes::application::resolve_files::{ResolveFiles, ResolveFilesRequest, ResolvedFile};
use jade_notes::application::save_attachment::{
    SaveAttachment, SaveAttachmentRequest, SavedAttachment,
};
use jade_notes::domain::model::{EntryKind, VaultEntry};
use jade_notes::domain::port::{DataFormatError, NoteRepositoryError};
use jade_notes::domain::value_object::{Data, VaultPath};
use jade_shared::application::use_case::UseCase;

use crate::support::mocks::{MockDataFormat, MockNoteRepository};
use crate::support::mothers::VAULT;

type Written = Arc<Mutex<Option<(String, Vec<u8>)>>>;

fn path(raw: &str) -> VaultPath {
    VaultPath::parse(raw).expect("valid path")
}

#[test]
fn resolves_each_target_to_a_vault_file() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_vault_lists(&mut repository, &["Recursos/Portada.webp"]);

    // Act
    let resolved = ResolveFiles::new(repository).execute(ResolveFilesRequest {
        vault_root: VAULT.to_string(),
        targets: vec!["Portada.webp".to_string(), "Nope.png".to_string()],
    });

    // Assert
    assert_eq!(
        resolved,
        Ok(vec![
            ResolvedFile {
                target: "Portada.webp".to_string(),
                path: Some(path("Recursos/Portada.webp"))
            },
            ResolvedFile {
                target: "Nope.png".to_string(),
                path: None
            },
        ])
    );
}

#[test]
fn saves_attachments_in_the_configured_folder_with_a_free_name() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_settings(&mut repository, "settings");
    given_existing(
        &mut repository,
        &["Recursos/foto.png", "Recursos/foto 1.png"],
    );
    let written = expect_written(&mut repository);
    given_the_vault_lists(
        &mut repository,
        &[
            "Recursos/foto.png",
            "Recursos/foto 1.png",
            "Recursos/foto 2.png",
        ],
    );

    // Act
    let saved = SaveAttachment::new(repository, given_the_documents_are_understood())
        .execute(request("Lore/Torre.md", "foto.png"));

    // Assert
    assert_eq!(
        saved,
        Ok(SavedAttachment {
            path: path("Recursos/foto 2.png"),
            link: "foto 2.png".to_string()
        })
    );
    assert_eq!(
        *written.lock().expect("lock"),
        Some(("Recursos/foto 2.png".to_string(), vec![1, 2, 3]))
    );
}

#[test]
fn links_by_path_when_the_name_is_ambiguous_and_saves_at_the_root_without_settings() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    repository
        .expect_read_text()
        .returning(|_, path| Err(NoteRepositoryError::NotFound(path.clone())));
    given_existing(&mut repository, &[]);
    let _written = expect_written(&mut repository);
    given_the_vault_lists(&mut repository, &["Otra/a/b/c/mapa", "mapa"]);

    // Act
    let saved =
        SaveAttachment::new(repository, MockDataFormat::new()).execute(request("Torre.md", "mapa"));

    // Assert
    assert_eq!(
        saved,
        Ok(SavedAttachment {
            path: path("mapa"),
            link: "mapa".to_string()
        })
    );
}

#[test]
fn links_by_path_when_another_file_takes_the_name() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_settings(&mut repository, "settings");
    given_existing(&mut repository, &[]);
    let _written = expect_written(&mut repository);
    given_the_vault_lists(&mut repository, &["Recursos/x.png", "x.png"]);

    // Act
    let saved = SaveAttachment::new(repository, given_the_documents_are_understood())
        .execute(request("Torre.md", "x.png"));

    // Assert
    assert_eq!(
        saved,
        Ok(SavedAttachment {
            path: path("Recursos/x.png"),
            link: "Recursos/x.png".to_string()
        })
    );
}

#[test]
fn ignores_malformed_settings() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_settings(&mut repository, "{broken");
    given_existing(&mut repository, &[]);
    let written = expect_written(&mut repository);
    given_the_vault_lists(&mut repository, &["foto.png"]);

    // Act
    let saved = SaveAttachment::new(repository, given_the_documents_are_understood())
        .execute(request("Lore/Torre.md", "foto.png"));

    // Assert
    assert!(saved.is_ok());
    assert_eq!(
        written
            .lock()
            .expect("lock")
            .as_ref()
            .map(|(path, _)| path.clone()),
        Some("foto.png".to_string())
    );
}

#[test]
fn rejects_unusable_names_and_notes() {
    // Arrange
    let requests = [
        request("Torre.md", " "),
        request("Torre.md", "a/b.png"),
        request("../x.md", "a.png"),
    ];

    // Act
    let results: Vec<_> = requests
        .into_iter()
        .map(|request| {
            SaveAttachment::new(MockNoteRepository::new(), MockDataFormat::new()).execute(request)
        })
        .collect();

    // Assert
    assert!(
        results
            .iter()
            .all(|result| matches!(result, Err(NotesError::InvalidPath(_)))),
        "{results:?}"
    );
}

#[test]
fn reports_write_failures() {
    // Arrange
    let mut repository = MockNoteRepository::new();
    given_the_settings(&mut repository, "settings");
    given_existing(&mut repository, &[]);
    repository
        .expect_write_bytes()
        .returning(|_, _, _| Err(NoteRepositoryError::Io("full".to_string())));

    // Act
    let saved = SaveAttachment::new(repository, given_the_documents_are_understood())
        .execute(request("Torre.md", "foto.png"));

    // Assert
    assert_eq!(saved, Err(NotesError::Storage("full".to_string())));
}

fn request(note: &str, file_name: &str) -> SaveAttachmentRequest {
    SaveAttachmentRequest {
        vault_root: VAULT.to_string(),
        note: note.to_string(),
        file_name: file_name.to_string(),
        content: vec![1, 2, 3],
    }
}

fn given_the_settings(repository: &mut MockNoteRepository, text: &'static str) {
    repository
        .expect_read_text()
        .returning(move |_, _| Ok(text.to_string()));
}

fn given_existing(repository: &mut MockNoteRepository, paths: &'static [&'static str]) {
    repository
        .expect_entry_exists()
        .returning(move |_, candidate| Ok(paths.contains(&candidate.as_str())));
}

fn expect_written(repository: &mut MockNoteRepository) -> Written {
    let written = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&written);
    repository
        .expect_write_bytes()
        .times(1)
        .returning(move |_, path, content| {
            *sink.lock().expect("lock") = Some((path.as_str().to_string(), content.to_vec()));
            Ok(())
        });
    written
}

fn given_the_vault_lists(repository: &mut MockNoteRepository, files: &'static [&'static str]) {
    repository.expect_list().returning(move |_| {
        Ok(files
            .iter()
            .map(|file| VaultEntry {
                path: VaultPath::parse(file).expect("valid"),
                kind: EntryKind::File,
            })
            .chain([VaultEntry {
                path: VaultPath::parse("Recursos").expect("valid"),
                kind: EntryKind::Folder,
            }])
            .collect())
    });
}

fn given_the_documents_are_understood() -> MockDataFormat {
    let mut format = MockDataFormat::new();
    format.expect_parse().returning(|text| match text {
        "settings" => Ok(Data::Map(vec![(
            "attachmentFolderPath".to_string(),
            Data::Text("Recursos".to_string()),
        )])),
        _ => Err(DataFormatError("malformed".to_string())),
    });
    format
}

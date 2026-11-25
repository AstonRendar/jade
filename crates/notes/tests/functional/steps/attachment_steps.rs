use cucumber::{then, when};
use jade_notes::application::resolve_files::{ResolveFiles, ResolveFilesRequest};
use jade_notes::application::save_attachment::{SaveAttachment, SaveAttachmentRequest};
use jade_notes::infrastructure::{FsNoteRepository, YamlDataFormat};
use jade_shared::application::use_case::UseCase;

use crate::world::NotesWorld;

#[when(expr = "the image {string} is attached to {string}")]
fn an_image_is_attached(world: &mut NotesWorld, name: String, note: String) {
    let saved = SaveAttachment::new(FsNoteRepository, YamlDataFormat)
        .execute(SaveAttachmentRequest {
            vault_root: world.root(),
            note,
            file_name: name,
            content: vec![0x89, b'P', b'N', b'G'],
        })
        .expect("the attachment is saved");
    world.attachment = Some((saved.path.as_str().to_string(), saved.link));
}

#[then(expr = "the attachment was saved as {string} and linked as {string}")]
fn the_attachment_was_saved(world: &mut NotesWorld, path: String, link: String) {
    assert_eq!(world.attachment, Some((path.clone(), link)));
    let bytes = std::fs::read(format!("{}/{path}", world.root())).expect("file exists");
    assert_eq!(bytes, vec![0x89, b'P', b'N', b'G']);
}

#[then(expr = "{string} resolves to the file {string}")]
fn a_target_resolves(world: &mut NotesWorld, target: String, path: String) {
    let resolved = ResolveFiles::new(FsNoteRepository)
        .execute(ResolveFilesRequest {
            vault_root: world.root(),
            targets: vec![target],
        })
        .expect("targets resolve");
    assert_eq!(
        resolved[0]
            .path
            .as_ref()
            .map(|found| found.as_str().to_string()),
        Some(path)
    );
}

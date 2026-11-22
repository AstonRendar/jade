use cucumber::{given, then, when};
use jade_notes::application::create_folder::{CreateFolder, CreateFolderRequest};
use jade_notes::application::create_note::{CreateNote, CreateNoteRequest};
use jade_notes::application::delete_entry::{DeleteEntry, DeleteEntryRequest};
use jade_notes::application::error::NotesError;
use jade_notes::application::list_vault_entries::{ListVaultEntries, ListVaultEntriesRequest};
use jade_notes::application::open_vault::{OpenVault, OpenVaultRequest};
use jade_notes::application::read_note::{ReadNote, ReadNoteRequest};
use jade_notes::application::rename_entry::{RenameEntry, RenameEntryRequest};
use jade_notes::application::save_note::{SaveNote, SaveNoteRequest};
use jade_notes::domain::model::{EntryKind, VaultEntry};
use jade_notes::domain::value_object::VaultPath;
use jade_notes::infrastructure::FsNoteRepository;
use jade_shared::application::use_case::UseCase;

use crate::world::NotesWorld;

#[given("an empty vault")]
fn an_empty_vault(_world: &mut NotesWorld) {}

#[when("the vault is opened")]
fn the_vault_is_opened(world: &mut NotesWorld) {
    let result = OpenVault::new(FsNoteRepository).execute(OpenVaultRequest {
        vault_root: world.root(),
    });
    world.opened = world.record(result);
}

#[when("a missing folder is opened as a vault")]
fn a_missing_folder_is_opened(world: &mut NotesWorld) {
    let result = OpenVault::new(FsNoteRepository).execute(OpenVaultRequest {
        vault_root: format!("{}/missing", world.root()),
    });
    world.opened = world.record(result);
}

#[then("the vault is named after its folder")]
fn the_vault_is_named_after_its_folder(world: &mut NotesWorld) {
    let opened = world.opened.as_ref().expect("vault was opened");
    assert_eq!(opened.name, world.folder_name());
}

#[then("the vault cannot be found")]
fn the_vault_cannot_be_found(world: &mut NotesWorld) {
    assert_eq!(world.last_error, Some(NotesError::VaultNotFound));
}

#[when(expr = "a note titled {string} is created")]
fn a_note_is_created(world: &mut NotesWorld, title: String) {
    create_note(world, None, title);
}

#[when(expr = "a note titled {string} is created in {string}")]
fn a_note_is_created_in_folder(world: &mut NotesWorld, title: String, folder: String) {
    create_note(world, Some(folder), title);
}

#[given(expr = "the folder {string} exists")]
fn the_folder_exists(world: &mut NotesWorld, path: String) {
    CreateFolder::new(FsNoteRepository)
        .execute(CreateFolderRequest {
            vault_root: world.root(),
            path,
        })
        .expect("folder is created");
}

#[given(expr = "the note {string} contains {string}")]
fn the_note_contains(world: &mut NotesWorld, path: String, content: String) {
    save(world, path, content.replace("\\n", "\n")).expect("note is saved");
}

#[given(expr = "the file {string} exists")]
fn the_file_exists(world: &mut NotesWorld, path: String) {
    std::fs::write(format!("{}/{path}", world.root()), [0u8, 1, 2]).expect("file is written");
}

#[when(expr = "{string} is saved with {string}")]
fn a_note_is_saved(world: &mut NotesWorld, path: String, content: String) {
    let result = save(world, path, content);
    world.record(result);
}

#[when(expr = "{string} is renamed to {string}")]
fn an_entry_is_renamed(world: &mut NotesWorld, from: String, to: String) {
    let result = RenameEntry::new(FsNoteRepository).execute(RenameEntryRequest {
        vault_root: world.root(),
        from,
        to,
    });
    world.record(result);
}

#[when(expr = "{string} is deleted")]
fn an_entry_is_deleted(world: &mut NotesWorld, path: String) {
    let result = DeleteEntry::new(FsNoteRepository).execute(DeleteEntryRequest {
        vault_root: world.root(),
        path,
    });
    world.record(result);
}

#[then(expr = "the vault lists the note {string}")]
fn the_vault_lists_the_note(world: &mut NotesWorld, path: String) {
    assert!(entries(world).contains(&entry(&path, EntryKind::Note)));
}

#[then(expr = "the vault lists the file {string}")]
fn the_vault_lists_the_file(world: &mut NotesWorld, path: String) {
    assert!(entries(world).contains(&entry(&path, EntryKind::File)));
}

#[then(expr = "the vault does not list {string}")]
fn the_vault_does_not_list(world: &mut NotesWorld, path: String) {
    assert!(entries(world).iter().all(|e| e.path.as_str() != path));
}

#[then(expr = "the vault lists exactly {string}")]
fn the_vault_lists_exactly(world: &mut NotesWorld, path: String) {
    assert_eq!(entries(world), vec![entry(&path, EntryKind::Note)]);
}

#[then(expr = "{string} is empty")]
fn the_note_is_empty(world: &mut NotesWorld, path: String) {
    assert_eq!(read(world, path), "");
}

#[then(expr = "{string} contains {string}")]
fn the_note_has_content(world: &mut NotesWorld, path: String, content: String) {
    assert_eq!(read(world, path), content);
}

#[then(expr = "the operation fails because {string} already exists")]
fn fails_because_it_exists(world: &mut NotesWorld, path: String) {
    let expected = VaultPath::parse(&path).expect("valid path");
    assert_eq!(world.last_error, Some(NotesError::AlreadyExists(expected)));
}

#[then("the operation fails because the move is not possible")]
fn fails_because_move_is_invalid(world: &mut NotesWorld) {
    assert_eq!(world.last_error, Some(NotesError::InvalidMove));
}

fn create_note(world: &mut NotesWorld, folder: Option<String>, title: String) {
    let result = CreateNote::new(FsNoteRepository).execute(CreateNoteRequest {
        vault_root: world.root(),
        folder,
        title,
    });
    world.record(result);
}

fn save(world: &NotesWorld, path: String, content: String) -> Result<(), NotesError> {
    SaveNote::new(FsNoteRepository).execute(SaveNoteRequest {
        vault_root: world.root(),
        path,
        content,
    })
}

fn read(world: &NotesWorld, path: String) -> String {
    ReadNote::new(FsNoteRepository)
        .execute(ReadNoteRequest {
            vault_root: world.root(),
            path,
        })
        .expect("note is read")
}

fn entries(world: &NotesWorld) -> Vec<VaultEntry> {
    ListVaultEntries::new(FsNoteRepository)
        .execute(ListVaultEntriesRequest {
            vault_root: world.root(),
        })
        .expect("entries are listed")
}

fn entry(path: &str, kind: EntryKind) -> VaultEntry {
    VaultEntry {
        path: VaultPath::parse(path).expect("valid path"),
        kind,
    }
}

#[when(expr = "a vault named {string} is created next to the current one")]
fn a_vault_is_created(world: &mut NotesWorld, name: String) {
    let result = create_vault(world, &name);
    world.opened = world.record(result);
    world.last_vault_name = Some(name);
}

#[then(expr = "the new vault is named {string}")]
fn the_new_vault_is_named(world: &mut NotesWorld, name: String) {
    let opened = world.opened.as_ref().expect("vault was created");
    assert_eq!(opened.name, name);
    assert!(opened.root.as_path().is_dir());
}

#[then("creating it again fails because it already exists")]
fn creating_again_fails(world: &mut NotesWorld) {
    let name = world.last_vault_name.clone().expect("a vault was created");
    let result = create_vault(world, &name);
    assert!(matches!(result, Err(NotesError::AlreadyExists(_))));
}

fn create_vault(
    world: &NotesWorld,
    name: &str,
) -> Result<jade_notes::application::open_vault::VaultInfo, NotesError> {
    let parent = std::path::Path::new(&world.root())
        .join("vaults")
        .display()
        .to_string();
    jade_notes::application::create_vault::CreateVault::new(FsNoteRepository).execute(
        jade_notes::application::create_vault::CreateVaultRequest {
            parent,
            name: name.to_string(),
        },
    )
}

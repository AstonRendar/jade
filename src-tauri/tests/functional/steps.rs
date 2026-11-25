use cucumber::{given, then, when};
use jade_lib::view_models::{
    CellView, ConflictCopyView, PropertyTypeView, PropertyValueView, PropertyView,
};

use crate::world::AppWorld;

#[given(expr = "a folder {string} with the note {string}")]
fn a_folder_with_a_note(world: &mut AppWorld, folder: String, note: String) {
    let path = world.path(&folder);
    std::fs::create_dir_all(&path).expect("folder is created");
    std::fs::write(path.join(note), "text").expect("note is written");
}

#[given(expr = "a remote vault {string} with {string} containing {string}")]
fn a_remote_vault(world: &mut AppWorld, name: String, note: String, content: String) {
    let remote = world.path(&format!("remotes/{name}.git"));
    let mut options = git2::RepositoryInitOptions::new();
    options.bare(true).initial_head("main");
    git2::Repository::init_opts(&remote, &options).expect("remote is created");
    world.remote = Some(remote);
    let seed = world.device_folder("seed").display().to_string();
    let url = world.remote_url();
    let app = world.app("seed");
    app.open_vault(&seed).expect("seed opens");
    app.init_repository().expect("seed is a repository");
    app.connect_remote(&url).expect("seed is connected");
    app.save_note(&note, &content).expect("note is saved");
    app.synchronize().expect("seed is pushed");
}

#[given(expr = "the app on {string} cloned the remote vault")]
#[when(expr = "the app on {string} clones the remote vault")]
fn the_app_clones_the_remote(world: &mut AppWorld, device: String) {
    let url = world.remote_url();
    clone(world, &device, &url);
}

#[when(expr = "the app on {string} clones {string}")]
fn the_app_clones_an_address(world: &mut AppWorld, device: String, url: String) {
    clone(world, &device, &url);
}

#[when(expr = "the app on {string} opens the folder {string}")]
fn the_app_opens_a_folder(world: &mut AppWorld, device: String, folder: String) {
    let path = world.path(&folder).display().to_string();
    let result = world.app(&device).open_vault(&path);
    if let Some(vault) = world.record(&device, result) {
        world.opened.insert(device, vault);
    }
}

#[when(expr = "the app on {string} lists the vault entries")]
fn the_app_lists_entries(world: &mut AppWorld, device: String) {
    let result = world.app(&device).list_entries();
    world.record(&device, result);
}

#[given(expr = "the app on {string} saves {string} with {string}")]
#[when(expr = "the app on {string} saves {string} with {string}")]
fn the_app_saves_a_note(world: &mut AppWorld, device: String, path: String, content: String) {
    world
        .app(&device)
        .save_note(&path, &content)
        .expect("note is saved");
}

#[given(expr = "the app on {string} synchronizes")]
#[when(expr = "the app on {string} synchronizes")]
fn the_app_synchronizes(world: &mut AppWorld, device: String) {
    let report = world.app(&device).synchronize().expect("sync succeeds");
    world.reports.insert(device, report);
}

#[when(
    expr = "the app on {string} sets the language {string}, theme {string} and sync every {int} minutes"
)]
fn the_app_sets_preferences(
    world: &mut AppWorld,
    device: String,
    locale: String,
    theme: String,
    minutes: u32,
) {
    world
        .app(&device)
        .update_preferences(Some(locale), &theme, minutes)
        .expect("preferences are updated");
}

#[when(expr = "the app on {string} stores the token {string}")]
fn the_app_stores_a_token(world: &mut AppWorld, device: String, token: String) {
    world
        .app(&device)
        .store_access_token(&token)
        .expect("token is stored");
}

#[then(expr = "the open vault on {string} is named {string}")]
fn the_open_vault_is_named(world: &mut AppWorld, device: String, name: String) {
    let opened = world.opened.get(&device).map(|vault| vault.name.as_str());
    assert_eq!(opened, Some(name.as_str()));
}

#[then(expr = "the most recent vault on {string} is the folder {string}")]
fn the_most_recent_vault_is(world: &mut AppWorld, device: String, folder: String) {
    let expected = world.path(&folder).display().to_string();
    let preferences = world.app(&device).preferences().expect("preferences load");
    assert_eq!(preferences.recent_vaults.first(), Some(&expected));
}

#[then(expr = "the vault on {string} lists {string}")]
fn the_vault_lists(world: &mut AppWorld, device: String, path: String) {
    let entries = world
        .app(&device)
        .list_entries()
        .expect("entries are listed");
    assert!(entries.iter().any(|entry| entry.path == path));
}

#[then(expr = "the app on {string} fails with {string}")]
fn the_app_fails_with(world: &mut AppWorld, device: String, code: String) {
    let error = world.errors.get(&device).expect("an error was recorded");
    let actual = serde_json::to_value(error.code).expect("code serialises");
    assert_eq!(actual, serde_json::Value::String(code));
}

#[then(expr = "the last sync on {string} reported the conflict copy {string} of {string}")]
fn reported_conflict_copy(world: &mut AppWorld, device: String, copy: String, original: String) {
    let report = world.reports.get(&device).expect("a sync ran");
    assert_eq!(
        report.conflict_copies,
        vec![ConflictCopyView { original, copy }]
    );
}

#[then(expr = "the repository on {string} has nothing pending")]
fn nothing_pending(world: &mut AppWorld, device: String) {
    let status = world
        .app(&device)
        .repository_status()
        .expect("status is read");
    assert!(status.is_repository);
    assert_eq!(
        (status.pending_changes, status.ahead, status.behind),
        (0, 0, 0)
    );
}

#[then(
    expr = "the preferences on {string} show language {string}, theme {string} and sync every {int} minutes"
)]
fn preferences_show(
    world: &mut AppWorld,
    device: String,
    locale: String,
    theme: String,
    minutes: u32,
) {
    let preferences = world.app(&device).preferences().expect("preferences load");
    assert_eq!(preferences.locale, Some(locale));
    assert_eq!(preferences.theme, theme);
    assert_eq!(preferences.auto_sync_minutes, minutes);
}

#[then(expr = "the app on {string} has a token")]
fn the_app_has_a_token(world: &mut AppWorld, device: String) {
    assert_eq!(world.app(&device).has_access_token(), Ok(true));
}

fn clone(world: &mut AppWorld, device: &str, url: &str) {
    let parent = world.device_folder(device).display().to_string();
    let result = world.app(device).clone_repository(url, &parent);
    if let Some(vault) = world.record(device, result) {
        world.opened.insert(device.to_string(), vault);
    }
}

#[given(expr = "the app on {string} opened the folder {string}")]
fn the_app_opened_a_folder(world: &mut AppWorld, device: String, folder: String) {
    let path = world.path(&folder).display().to_string();
    world.app(&device).open_vault(&path).expect("vault opens");
}

#[when(expr = "the app on {string} creates the note {string} in the root")]
fn the_app_creates_a_note(world: &mut AppWorld, device: String, title: String) {
    world
        .app(&device)
        .create_note(None, &title)
        .expect("note is created");
}

#[when(expr = "the app on {string} creates the folder {string}")]
fn the_app_creates_a_folder(world: &mut AppWorld, device: String, path: String) {
    world
        .app(&device)
        .create_folder(&path)
        .expect("folder is created");
}

#[when(expr = "the app on {string} renames {string} to {string}")]
fn the_app_renames(world: &mut AppWorld, device: String, from: String, to: String) {
    world
        .app(&device)
        .rename_entry(&from, &to)
        .expect("entry is renamed");
}

#[when(expr = "the app on {string} follows the link {string}")]
fn the_app_follows_a_link(world: &mut AppWorld, device: String, target: String) {
    world
        .app(&device)
        .follow_link(&target)
        .expect("link is followed");
}

#[when(expr = "the app on {string} deletes {string}")]
fn the_app_deletes(world: &mut AppWorld, device: String, path: String) {
    world
        .app(&device)
        .delete_entry(&path)
        .expect("entry is deleted");
}

#[when(expr = "the app on {string} initialises a repository")]
fn the_app_initialises(world: &mut AppWorld, device: String) {
    world
        .app(&device)
        .init_repository()
        .expect("repository is initialised");
}

#[when(expr = "the app on {string} forgets the folder {string}")]
fn the_app_forgets(world: &mut AppWorld, device: String, folder: String) {
    let path = world.path(&folder).display().to_string();
    world
        .app(&device)
        .forget_vault(&path)
        .expect("vault is forgotten");
}

#[then(expr = "{string} on {string} reads {string}")]
fn a_note_reads(world: &mut AppWorld, path: String, device: String, content: String) {
    assert_eq!(world.app(&device).read_note(&path), Ok(content));
}

#[then(expr = "searching {string} on {string} finds {string}")]
fn searching_finds(world: &mut AppWorld, query: String, device: String, path: String) {
    let hits = world.app(&device).search(&query).expect("search runs");
    assert!(hits.iter().any(|hit| hit.path == path));
}

#[then(expr = "the backlinks of {string} on {string} come from {string}")]
fn backlinks_come_from(world: &mut AppWorld, path: String, device: String, source: String) {
    let backlinks = world.app(&device).backlinks(&path).expect("backlinks load");
    let sources: Vec<&str> = backlinks.iter().map(|b| b.source.as_str()).collect();
    assert_eq!(sources, vec![source.as_str()]);
}

#[then(expr = "the link {string} on {string} resolves to {string}")]
fn the_link_resolves(world: &mut AppWorld, target: String, device: String, path: String) {
    let resolved = world
        .app(&device)
        .resolve_links(vec![target])
        .expect("links resolve");
    assert_eq!(resolved[0].path.as_deref(), Some(path.as_str()));
}

#[then(expr = "the vault on {string} does not list {string}")]
fn the_vault_does_not_list(world: &mut AppWorld, device: String, path: String) {
    let entries = world
        .app(&device)
        .list_entries()
        .expect("entries are listed");
    assert!(entries.iter().all(|entry| entry.path != path));
}

#[then(expr = "the history of {string} on {string} has {int} revision(s)")]
fn history_has(world: &mut AppWorld, path: String, device: String, count: usize) {
    let history = world
        .app(&device)
        .note_history(&path)
        .expect("history loads");
    assert_eq!(history.len(), count);
}

#[then(expr = "the first revision of {string} on {string} reads {string}")]
fn first_revision_reads(world: &mut AppWorld, path: String, device: String, content: String) {
    let app = world.app(&device);
    let history = app.note_history(&path).expect("history loads");
    let oldest = history.last().expect("history is not empty");
    assert_eq!(app.note_revision(&path, &oldest.id), Ok(content));
}

#[then(expr = "{string} remembers no vaults")]
fn remembers_no_vaults(world: &mut AppWorld, device: String) {
    let preferences = world.app(&device).preferences().expect("preferences load");
    assert!(preferences.recent_vaults.is_empty());
}

#[when(expr = "the app on {string} creates the vault {string} in the folder {string}")]
fn the_app_creates_a_vault(world: &mut AppWorld, device: String, name: String, folder: String) {
    let parent = world.path(&folder).display().to_string();
    let result = world.app(&device).create_vault(&parent, &name);
    if let Some(vault) = world.record(&device, result) {
        world.opened.insert(device, vault);
    }
}

#[when(expr = "the app on {string} deletes its vault {string}")]
fn the_app_deletes_its_vault(world: &mut AppWorld, device: String, name: String) {
    let container = world.device_folder(&device);
    let vault = container.join(name).display().to_string();
    let result = world
        .app(&device)
        .delete_vault(&vault, Some(&container.display().to_string()));
    world.record(&device, result);
}

#[when(expr = "the app on {string} deletes the folder {string}")]
fn the_app_deletes_a_folder(world: &mut AppWorld, device: String, folder: String) {
    let container = world.device_folder(&device).display().to_string();
    let vault = world.path(&folder).display().to_string();
    let result = world.app(&device).delete_vault(&vault, Some(&container));
    world.record(&device, result);
}

#[then(expr = "the vault {string} is gone from {string}")]
fn the_vault_is_gone(world: &mut AppWorld, name: String, device: String) {
    assert!(!world.device_folder(&device).join(name).exists());
}

#[then(expr = "the app on {string} remembers no vaults")]
fn the_app_remembers_no_vaults(world: &mut AppWorld, device: String) {
    let preferences = world.app(&device).preferences().expect("preferences load");
    assert!(preferences.recent_vaults.is_empty());
}

#[then(expr = "listing the entries on {string} fails with {string}")]
fn listing_fails_with(world: &mut AppWorld, device: String, code: String) {
    let error = world
        .app(&device)
        .list_entries()
        .expect_err("listing fails");
    let actual = serde_json::to_value(error.code).expect("code serialises");
    assert_eq!(actual, serde_json::Value::String(code));
}

#[then(expr = "the folder {string} still exists")]
fn the_folder_still_exists(world: &mut AppWorld, folder: String) {
    assert!(world.path(&folder).is_dir());
}

#[given(expr = "the folder {string} has the file {string} with {string}")]
fn the_folder_has_a_file(world: &mut AppWorld, folder: String, file: String, content: String) {
    let path = world.path(&folder).join(file);
    std::fs::write(path, content.replace("\\n", "\n")).expect("file is written");
}

#[when(expr = "the app on {string} queries the base {string}")]
fn the_app_queries_a_base(world: &mut AppWorld, device: String, path: String) {
    let result = world.app(&device).query_base(&path, None);
    world.record(&device, result);
}

#[then(expr = "the base {string} on {string} shows {string} with {string}")]
fn the_base_shows(world: &mut AppWorld, path: String, device: String, row: String, cells: String) {
    let table = world
        .app(&device)
        .query_base(&path, None)
        .expect("the base is queried");
    let found = table
        .rows
        .iter()
        .find(|candidate| candidate.path == row)
        .expect("the row is shown");
    let actual: Vec<String> = found.cells.iter().map(cell_text).collect();
    assert_eq!(actual.join(" | "), cells);
}

fn cell_text(cell: &CellView) -> String {
    match cell {
        CellView::Text { value } => value.clone(),
        CellView::List { items } => items.iter().map(cell_text).collect::<Vec<_>>().join(", "),
        CellView::Link { target, .. } => target.clone(),
        other => format!("{other:?}"),
    }
}

#[when(expr = "the app on {string} sets the property {string} of {string} to true")]
fn the_app_sets_a_property(world: &mut AppWorld, device: String, key: String, path: String) {
    let app = world.app(&device);
    let content = app.read_note(&path).expect("note is read");
    let mut properties = app
        .read_properties(content.clone())
        .expect("properties are read")
        .properties;
    properties.push(PropertyView {
        key,
        value: PropertyValueView::Boolean { value: true },
    });
    let written = app
        .write_properties(content, properties)
        .expect("properties are written");
    app.save_note(&path, &written).expect("note is saved");
}

#[then(expr = "{string} on {string} has the properties {string} and the body {string}")]
fn a_note_has_properties(
    world: &mut AppWorld,
    path: String,
    device: String,
    keys: String,
    body: String,
) {
    let app = world.app(&device);
    let content = app.read_note(&path).expect("note is read");
    let read = app
        .read_properties(content.clone())
        .expect("properties are read");
    let actual: Vec<String> = read
        .properties
        .iter()
        .map(|property| property.key.clone())
        .collect();
    assert_eq!(actual.join(", "), keys);
    let units: Vec<u16> = content.encode_utf16().collect();
    let start = usize::try_from(read.body_start).expect("fits");
    assert_eq!(String::from_utf16_lossy(&units[start..]), body);
}

#[when(expr = "the app on {string} makes {string} a list")]
fn the_app_makes_a_list(world: &mut AppWorld, device: String, name: String) {
    world
        .app(&device)
        .set_property_type(&name, "multitext")
        .expect("the type is stored");
}

#[then(expr = "the vault on {string} knows the property {string} as a {word}")]
fn the_vault_knows_a_property(world: &mut AppWorld, device: String, name: String, kind: String) {
    let summaries = world
        .app(&device)
        .list_properties()
        .expect("properties are listed");
    let summary = summaries
        .iter()
        .find(|summary| summary.name == name)
        .expect("the property is listed");
    let expected = match kind.as_str() {
        "checkbox" => PropertyTypeView::Checkbox,
        _ => PropertyTypeView::List,
    };
    assert_eq!(summary.kind, expected);
}
